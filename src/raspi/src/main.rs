use std::time::Instant;
use serde::Serialize;
use warp::{reply::json, Reply, Filter, ws::{WebSocket, Message}};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    
    let health_route = warp::path("health").and_then(move || health_handler(start_time));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and_then(ws_handler);

    let frontend = warp::path::end().and(warp::fs::file("assets/GUI-HTML.html"));
    let assets = warp::path("assets").and(warp::fs::dir("assets"));
    
    let routes = health_route
        .or(ws_route)
        .or(assets)
        .or(frontend)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0, 0, 0, 0], 6060)).await;
}

pub async fn ws_handler(ws: warp::ws::Ws) -> Result<impl Reply, warp::Rejection> {
    Ok(ws.on_upgrade(websocket_connection))
}

pub async fn websocket_connection(ws: WebSocket) {
    let (mut client_ws_sender, mut client_ws_rcv) = ws.split();

    while let Some(result) = client_ws_rcv.next().await {
        let msg_raw = match result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error receiving ws message: {}", e);
                break;
            },
        };

        let msg_text = match msg_raw.to_str() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error receiving ws message: {:?}", e);
                continue;
            },
        };

        println!("Got {}", msg_text);

        // let msg: BasePacket = conerr!(serde_json::from_str(&msg_text), e, {
        //     eprintln!("error parsing base packet: {}", e);
        // });
    
        // let mut state_guard = state_mutex.lock().await;
        // let state = state_guard.deref_mut();
        // handle_packet(&mut session_handle_o, msg, &mut send_channel, state)
    }
}

// pub async fn ws_handler(ws: warp::ws::Ws, state_mutex: Arc<Mutex<VTTState>>) -> Result<impl Reply, warp::Rejection> {
//     Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, state_mutex)))
// }

#[derive(Serialize)]
pub struct HealthResponse {
    uptime: String,
}

pub async fn health_handler(start_time: std::time::Instant) -> Result<impl Reply, warp::Rejection> {
    return Ok(json(&HealthResponse {
        uptime: format!("{} s", start_time.elapsed().as_secs_f32()),
    }));
}