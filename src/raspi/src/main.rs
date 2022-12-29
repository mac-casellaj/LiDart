use serde::Serialize;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Instant;
use warp::{reply::json, Filter, Reply};
use std::collections::HashMap;
use crate::defs::State;

mod defs;
mod drive;
mod video;

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(State{
        start_time: Instant::now(),
        port: None,

        viddown_conn_i: 0,
        viddown_conns: HashMap::new(),

        vidstate_conn_i: 0,
        vidstate_conns: HashMap::new(),

        curr_frame: Vec::new(),
    }));

    let health_route_state = shared_state.clone();
    let health_route = warp::path("health").then(move || health_handler(health_route_state.clone()));

    async fn ws_handler(ws: warp::ws::Ws, shared_state: Arc<Mutex<State>>) -> impl Reply {
        ws.on_upgrade(move |socket| drive::websocket_connection(socket, shared_state))
    }

    let ws_route_state = shared_state.clone();
    let ws_route = warp::path("ws").and(warp::ws()).then(move |ws| ws_handler(ws, ws_route_state.clone()));

    async fn vidup_handler(ws: warp::ws::Ws, shared_state: Arc<Mutex<State>>) -> impl Reply {
        ws.on_upgrade(move |socket| video::vidup_connection(socket, shared_state))
    }

    let vidup_route_state = shared_state.clone();
    let vidup_route = warp::path("vidup").and(warp::ws()).then(move |ws| vidup_handler(ws, vidup_route_state.clone()));

    async fn viddown_handler(ws: warp::ws::Ws, shared_state: Arc<Mutex<State>>) -> impl Reply {
        ws.on_upgrade(move |socket| video::viddown_connection(socket, shared_state))
    }

    let viddown_route_state = shared_state.clone();
    let viddown_route = warp::path("viddown").and(warp::ws()).then(move |ws| viddown_handler(ws, viddown_route_state.clone()));

    async fn vidstate_handler(ws: warp::ws::Ws, shared_state: Arc<Mutex<State>>) -> impl Reply {
        ws.on_upgrade(move |socket| video::vidstate_connection(socket, shared_state))
    }

    let vidstate_route_state = shared_state.clone();
    let vidstate_route = warp::path("vidstate").and(warp::ws()).then(move |ws| vidstate_handler(ws, vidstate_route_state.clone()));

    let frontend = warp::path::end().and(warp::fs::file("assets/GUI-HTML.html"));
    let assets = warp::path("assets").and(warp::fs::dir("assets"));

    let routes = health_route
        .or(ws_route)
        .or(vidup_route)
        .or(viddown_route)
        .or(vidstate_route)
        .or(assets)
        .or(frontend)
        .with(warp::cors().allow_any_origin());

    drive::start_serial_reader(shared_state);
    warp::serve(routes).run(([0, 0, 0, 0], 6060)).await;
}

#[derive(Serialize)]
pub struct HealthResponse {
    uptime: String,
}

pub async fn health_handler(shared_state: Arc<Mutex<State>>) -> impl Reply {
    let state = shared_state.lock().await;
    
    return json(&HealthResponse {
        uptime: format!("{} s", state.start_time.elapsed().as_secs_f32()),
    });
}
