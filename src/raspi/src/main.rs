use futures::StreamExt;
use serde::Serialize;
use serialport::SerialPort;
use tokio::{sync::Mutex, task, time::sleep};
use warp::hyper::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use warp::{reply::json, ws::WebSocket, Filter, Reply};

pub struct State {
    start_time: Instant,
    port: Option<Box<dyn SerialPort>>,
}

impl State {
    fn serial_write(&mut self, data: &[u8]) {
        let port = match &mut self.port {
            Some(port) => port,
            None => {
                println!("No serial port connected, listing available ports");
                match serialport::available_ports() {
                    Ok(available_ports) => {
                        for available_port in available_ports {
                            if !matches!(available_port.port_type, serialport::SerialPortType::UsbPort(_)) { continue; }
                            print!("Attempting to connect to {} {:?} ... ", available_port.port_name, available_port.port_type);

                            let attemped_port = serialport::new(available_port.port_name, 9600)
                                .timeout(Duration::from_millis(10))
                                .open();

                            match attemped_port {
                                Ok(v) => {
                                    println!("Success!");
                                    self.port = Some(v);
                                    break;
                                },
                                Err(e) => println!("Failure: {}", e),
                            }
                        }

                        match &mut self.port {
                            Some(port) => port,
                            None => {
                                println!("Failed to find a connection");
                                return;
                            }
                        }
                    },
                    Err(e) => {
                        println!("Error listing available ports: {}", e);
                        return;
                    },
                }
            },
        };

        if let Err(e) = port.write(data) {
            println!("Error writing to serial, dropping connection: {}", e);
            self.port = None;
        }
    } 
}

#[derive(Serialize)]
pub struct ApriltagsDetection {
    corners: [[f64; 2]; 4],
}

fn handle_apriltags(bytes: bytes::Bytes) -> Box<dyn Reply> {
    let family = apriltag::families::Family::tag_36h11();
    let tag_params = apriltag::pose::TagParams{
        tagsize: 0.0,
        fx: 0.0,
        fy: 0.0,
        cx: 0.0,
        cy: 0.0,
    };

    let mut detector = apriltag::detector::DetectorBuilder::new()
        .add_family_bits(family, 1)
        .build()
        .unwrap();

    let src_image = match image::load_from_memory(&bytes) {
        Ok(v) => v.to_luma8(),
        Err(e) => {
            eprintln!("handle_apriltags: {}", e);
            return Box::new(StatusCode::BAD_REQUEST);
        },
    };

    const DEFAULT_ALIGNMENT_U8: usize = 96;
    use image::Pixel;

    let width = src_image.width() as usize;
    let height = src_image.height() as usize;
    let mut image = apriltag::Image::zeros_alignment(width, height, DEFAULT_ALIGNMENT_U8).unwrap();

    src_image.enumerate_pixels().for_each(|(x, y, pixel)| {
        let component = pixel.channels()[0];
        image[(x as usize, y as usize)] = component;
    });

    let detections = detector.detect(image);

    // TODO(Jon): trait implementation to convert from image's luma8 to apriltag's image
    //            seems broken
    // let detections = detector.detect(src_image);

    // println!("= image {}", path.display());

    let response: Vec<ApriltagsDetection> = detections.into_iter().enumerate().map(|(index, det)| {
        // println!("  - detection {}: {:#?}", index, det);
        // if let Some(tag_params) = &tag_params {
        //     let pose = det.estimate_tag_pose(tag_params);
        //     println!("  - pose {}: {:#?}", index, pose);

        //     let isometry = pose.map(|pose| pose.to_isometry());
        //     println!("  - isometry {}: {:#?}", index, isometry);
        // }
        ApriltagsDetection {
            corners: det.corners(),
        }
    }).collect();
    
    Box::new(json(&response))
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(State{
        start_time: Instant::now(),
        port: None,
    }));

    let health_route_state = shared_state.clone();
    let health_route = warp::path("health").and_then(move || health_handler(health_route_state.clone()));

    let ws_route_state = shared_state.clone();
    let ws_route = warp::path("ws").and(warp::ws()).and_then(move |ws| ws_handler(ws, ws_route_state.clone()));

    let apriltags_route = warp::path("apriltags")
        .and(warp::body::content_length_limit(1024 * 128))
        .and(warp::body::bytes())
        .map(handle_apriltags);

    let frontend = warp::path::end().and(warp::fs::file("assets/GUI-HTML.html"));
    let assets = warp::path("assets").and(warp::fs::dir("assets"));

    let routes = health_route
        .or(ws_route)
        .or(apriltags_route)
        .or(assets)
        .or(frontend)
        .with(warp::cors().allow_any_origin());

    // background task to read from port
    task::spawn(async move {
        let mut read_buf_used: usize = 128;
        let mut read_buf: [u8; 512] = [0; 512];
        loop {
            {
                let mut state = shared_state.lock().await;

                if let Some(port) = &mut state.port {
                    match port.read(&mut read_buf[read_buf_used..]) {
                        Ok(n) => {
                            println!("Got {} bytes", n);

                            let old_used = read_buf_used;
                            read_buf_used += n;
                            let mut chars_consumed = 0;

                            // print complete lines from read buf
                            for i in old_used..read_buf_used {
                                if (read_buf[i] as char) == '\n' {
                                    for j in chars_consumed..(i+1) {
                                        print!("{}", read_buf[j] as char);        
                                    }
                                    chars_consumed = i + 1;
                                }
                            }

                            // shift read_buf back
                            for i in 0..(read_buf_used - chars_consumed) {
                                read_buf[i] = read_buf[i + chars_consumed];
                            }

                            read_buf_used -= chars_consumed;
                        },
                        Err(_e) => {
                            // println!("Error reading from port, dropping connection: {}", e);
                            state.port = None;
                        }
                    }
                }
            }

            sleep(Duration::from_millis(100)).await;    
        }
    });

    warp::serve(routes).run(([0, 0, 0, 0], 6060)).await;
}

pub async fn ws_handler(ws: warp::ws::Ws, shared_state: Arc<Mutex<State>>) -> Result<impl Reply, warp::Rejection> {
    Ok(ws.on_upgrade(move |socket| websocket_connection(socket, shared_state)))
}

pub fn make_serial_packet(left_speed: u8, right_speed: u8) -> [u8; 8] {
    [0x10, 0x02, 0x02, left_speed, right_speed, left_speed ^ right_speed, 0x10, 0x03]
}

pub async fn websocket_connection(ws: WebSocket, shared_state: Arc<Mutex<State>>) {
    let (mut _client_ws_sender, mut client_ws_rcv) = ws.split();

    while let Some(result) = client_ws_rcv.next().await {
        let msg_raw = match result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error receiving ws message: {}", e);
                break;
            }
        };

        let msg_text = match msg_raw.to_str() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error receiving ws message: {:?}", e);
                continue;
            }
        };

        println!("Got {}", msg_text);

        let mut state = shared_state.lock().await;
        let serial_data = match msg_text {
            "FORWARD" => make_serial_packet(255, 255),
            "BACK" => make_serial_packet(127, 127),
            "LEFT" => make_serial_packet(127, 255),
            "RIGHT" => make_serial_packet(255, 127),
            "STOP" => make_serial_packet(0, 0),
            _ => continue,
        };

        state.serial_write(&serial_data);
    }
}

#[derive(Serialize)]
pub struct HealthResponse {
    uptime: String,
}

pub async fn health_handler(shared_state: Arc<Mutex<State>>) -> Result<impl Reply, warp::Rejection> {
    let state = shared_state.lock().await;
    
    return Ok(json(&HealthResponse {
        uptime: format!("{} s", state.start_time.elapsed().as_secs_f32()),
    }));
}
