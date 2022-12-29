use apriltag::{Family, DetectorBuilder};
use futures::{StreamExt, SinkExt};
use serde::Serialize;
use tokio::sync::{mpsc, Mutex};
use std::{sync::Arc, time::Instant};
use warp::ws::{WebSocket, Message};
use crate::defs::{State, TAG_PARAMS};

#[derive(Serialize)]
pub struct ApriltagDetection {
    id: usize,
    center: [f64; 2],
    corners: [[f64; 2]; 4],
}

pub fn detect_apriltags(family: fn() -> Family, data: &[u8]) -> Vec<ApriltagDetection> {
    let before = Instant::now();
    let mut detector = DetectorBuilder::new()
        .add_family_bits(family(), 1)
        .build()
        .unwrap();
    println!("Detector setup took {:#?}", before.elapsed());

    let before = Instant::now();
    let src_image = match image::load_from_memory(data) {
        Ok(v) => v.to_luma8(),
        Err(e) => {
            eprintln!("handle_apriltags: {}", e);
            return Vec::new();
        },
    };
    println!("Image decode took {:#?}", before.elapsed());

    const DEFAULT_ALIGNMENT_U8: usize = 96;
    use image::Pixel;

    let before = Instant::now();
    let width = src_image.width() as usize;
    let height = src_image.height() as usize;
    let mut image = apriltag::Image::zeros_alignment(width, height, DEFAULT_ALIGNMENT_U8).unwrap();

    src_image.enumerate_pixels().for_each(|(x, y, pixel)| {
        let component = pixel.channels()[0];
        image[(x as usize, y as usize)] = component;
    });
    println!("Image conversion took {:#?}", before.elapsed());

    let before = Instant::now();
    let detections = detector.detect(image);
    println!("Detection took {:#?}", before.elapsed());

    // TODO(Jon): trait implementation to convert from image's luma8 to apriltag's image
    //            seems broken
    // let detections = detector.detect(src_image);

    // println!("= image {}", path.display());

    detections.into_iter().map(|det| {
        println!("Detection: {:#?}", det);
        
        let before = Instant::now();
        if let Some(pose) = det.estimate_tag_pose(&TAG_PARAMS) {
            println!("Pose: {:#?}", pose);
        } else {
            println!("Pose estimation failed");
        }
        println!("Pose estimation took {:#?}", before.elapsed());
        
        println!();
        
        ApriltagDetection {
            id: det.id(),
            center: det.center(),
            corners: det.corners(),
        }
    }).collect()
}

pub async fn vidup_connection(ws: WebSocket, shared_state: Arc<Mutex<State>>) {
    let (_client_ws_sender, mut client_ws_rcv) = ws.split();
    
    println!("[vidup conn] Recv loop started");

    while let Some(result) = client_ws_rcv.next().await {
        let msg_raw = match result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[vidup conn] Error receiving ws message: {}", e);
                break;
            }
        };

        let msg_bytes = msg_raw.into_bytes();

        let mut state = shared_state.lock().await;
        for conn in state.viddown_conns.values() {
            let _ = conn.send(msg_bytes.clone());
        }

        state.curr_frame = msg_bytes;
    }
    
    println!("[vidup conn] Recv loop ended");
}

pub async fn viddown_connection(ws: WebSocket, shared_state: Arc<Mutex<State>>) {
    let (mut client_ws_sender, _client_ws_rcv) = ws.split();
    let (client_sender, mut client_rcv) = mpsc::unbounded_channel();

    let conn_i;
    {
        let mut state = shared_state.lock().await;
        conn_i = state.viddown_conn_i;
        state.viddown_conns.insert(conn_i, client_sender);
        state.viddown_conn_i += 1;
    }

    println!("[viddown conn {}] Send loop started", conn_i);

    while let Some(to_send) = client_rcv.recv().await {
        if let Err(e) = client_ws_sender.send(Message::binary(to_send)).await {
            eprintln!("[viddown conn {}] Error sending websocket msg: {}", conn_i, e);
            break;
        }
    }
    
    if let Err(e) = client_ws_sender.close().await {
        eprintln!("[viddown conn {}] Error closing websocket: {}", conn_i, e);
    }
    
    println!("[viddown conn {}] Send loop ended", conn_i);

    {
        let mut state = shared_state.lock().await;
        state.viddown_conns.remove(&conn_i);
    }
}

pub async fn vidstate_connection(ws: WebSocket, shared_state: Arc<Mutex<State>>) {
    let (mut client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, mut client_rcv) = mpsc::unbounded_channel();

    let conn_i;
    {
        let mut state = shared_state.lock().await;
        conn_i = state.vidstate_conn_i;
        state.vidstate_conns.insert(conn_i, client_sender);
        state.vidstate_conn_i += 1;
    }

    // Background send loop
    tokio::task::spawn(async move {
        println!("[vidstate conn {}] Send loop started", conn_i);

        while let Some(to_send) = client_rcv.recv().await {
            if let Err(e) = client_ws_sender.send(Message::text(to_send)).await {
                eprintln!("[vidstate conn {}] Error sending websocket msg: {}", conn_i, e);
            }
        }
        
        if let Err(e) = client_ws_sender.close().await {
            eprintln!("[vidstate conn {}] Error closing websocket: {}", conn_i, e);
        }
        
        println!("[vidstate conn {}] Send loop ended", conn_i);
    });

    println!("[vidstate conn {}] Recv loop started", conn_i);
    
    while let Some(result) = client_ws_rcv.next().await {
        let msg_raw = match result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[vidstate conn {}] Error receiving ws message: {}", conn_i, e);
                break;
            }
        };

        let msg_text = match msg_raw.to_str() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[vidstate conn {}] Error receiving ws message: {:?}", conn_i, e);
                continue;
            }
        };

        println!("[vidstate conn {}] got {}", conn_i, msg_text);

        // let msg: BasePacket = match serde_json::from_str(&msg_text) {
        //     Ok(v) => v,
        //     Err(e) => {
        //         eprintln!("[vidstate conn {}] Error parsing base packet: {}", conn_i, e);
        //         continue;
        //     },
        // };

        if msg_text == "DETECT" {
            let state = shared_state.lock().await;

            let before = Instant::now();
            let detections = detect_apriltags(Family::tag_36h11, &state.curr_frame);
            
            let detections_json = match serde_json::to_string(&detections) {
                Ok(v) => v,
                Err(e) => {
                    println!("[vidup conn] Error encoding apriltag detections to json: {}", e); 
                    continue;
                },
            };
    
            for conn in state.vidstate_conns.values() {
                let _ = conn.send(detections_json.clone());
            }
            println!("Entire detect request took {:#?}", before.elapsed());
        }
    }
    
    println!("[vidstate conn {}] Recv loop ended", conn_i);

    {
        let mut state = shared_state.lock().await;
        state.vidstate_conns.remove(&conn_i);
    }
}