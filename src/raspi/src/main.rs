use futures::StreamExt;
use serde::Serialize;
use serialport::SerialPort;
use tokio::sync::Mutex;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use warp::{reply::json, ws::WebSocket, Filter, Reply};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Invalid params {:#?}", args);
        return;
    }

    let start_time = Instant::now();
    let health_route = warp::path("health").and_then(move || health_handler(start_time));

    // Open the serial port
    // Defining the serial port for communication
    let port = serialport::new(&args[1], 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let shared_port = Arc::new(Mutex::new(port));

    let ws_route = warp::path("ws").and(warp::ws()).and_then(move |ws| ws_handler(ws, shared_port.clone()));

    let frontend = warp::path::end().and(warp::fs::file("assets/GUI-HTML.html"));
    let assets = warp::path("assets").and(warp::fs::dir("assets"));

    let routes = health_route
        .or(ws_route)
        .or(assets)
        .or(frontend)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0, 0, 0, 0], 6060)).await;
}

pub async fn ws_handler(ws: warp::ws::Ws, shared_port: Arc<Mutex<Box<dyn SerialPort>>>) -> Result<impl Reply, warp::Rejection> {
    Ok(ws.on_upgrade(move |socket| websocket_connection(socket, shared_port)))
}

pub fn make_serial_packet(left_speed: u8, right_speed: u8) -> [u8; 8] {
    return [0x10, 0x02, 0x02, left_speed, right_speed, left_speed ^ right_speed, 0x10, 0x03];
}

pub async fn websocket_connection(ws: WebSocket, shared_port: Arc<Mutex<Box<dyn SerialPort>>>) {
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

        let mut port = shared_port.lock().await;
        let serial_data = match msg_text {
            "FORWARD" => make_serial_packet(255, 255),
            "BACK" => make_serial_packet(127, 127),
            "LEFT" => make_serial_packet(127, 255),
            "RIGHT" => make_serial_packet(255, 127),
            "STOP" => make_serial_packet(0, 0),
            _ => continue,
        };

        let _ = port.write(&serial_data);
    }
}

#[derive(Serialize)]
pub struct HealthResponse {
    uptime: String,
}

pub async fn health_handler(start_time: std::time::Instant) -> Result<impl Reply, warp::Rejection> {
    return Ok(json(&HealthResponse {
        uptime: format!("{} s", start_time.elapsed().as_secs_f32()),
    }));
}
