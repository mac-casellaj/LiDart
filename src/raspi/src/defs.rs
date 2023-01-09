use apriltag::TagParams;
use serialport::SerialPort;
use std::time::Instant;
use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::math::{Mat3, Vec3};

pub struct Pose {
    pub rotation: Mat3,
    pub translation: Vec3,
}

pub struct State {
    pub start_time: Instant,
    pub port: Option<Box<dyn SerialPort>>,

    pub viddown_conn_i: u32,
    pub viddown_conns: HashMap<u32, mpsc::UnboundedSender<Vec<u8>>>,

    pub vidstate_conn_i: u32,
    pub vidstate_conns: HashMap<u32, mpsc::UnboundedSender<String>>,

    pub curr_frame: Vec<u8>,

    pub detected_landmarks: HashMap<usize, Pose>,
}

pub const TAG_PARAMS: TagParams = TagParams{
    tagsize: 0.089, // ~89mm
    fx: 253.68282598,
    fy: 253.5225799,
    cx: 240.19900698,
    cy: 319.43963706,
};