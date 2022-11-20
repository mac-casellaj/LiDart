use futures::StreamExt;
use serde::Serialize;
use serialport::SerialPort;
use tokio::sync::Mutex;
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

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State{
        start_time: Instant::now(),
        port: None,
    }));

    let health_route_state = state.clone();
    let health_route = warp::path("health").and_then(move || health_handler(health_route_state.clone()));

    let ws_route = warp::path("ws").and(warp::ws()).and_then(move |ws| ws_handler(ws, state.clone()));

    let frontend = warp::path::end().and(warp::fs::file("assets/GUI-HTML.html"));
    let assets = warp::path("assets").and(warp::fs::dir("assets"));

    let routes = health_route
        .or(ws_route)
        .or(assets)
        .or(frontend)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0, 0, 0, 0], 6060)).await;
}

pub async fn ws_handler(ws: warp::ws::Ws, shared_state: Arc<Mutex<State>>) -> Result<impl Reply, warp::Rejection> {
    Ok(ws.on_upgrade(move |socket| websocket_connection(socket, shared_state)))
}

pub fn make_serial_packet(left_speed: u8, right_speed: u8) -> [u8; 8] {
    return [0x10, 0x02, 0x02, left_speed, right_speed, left_speed ^ right_speed, 0x10, 0x03];
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
