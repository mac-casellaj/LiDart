use futures::StreamExt;
use tokio::{sync::Mutex, task, time::sleep};
use std::sync::Arc;
use std::time::Duration;
use warp::ws::WebSocket;

use crate::defs::State;

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

// background task to read from port
pub fn start_serial_reader(shared_state: Arc<Mutex<State>>) {
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
}

pub fn make_serial_packet(left_speed: u8, right_speed: u8) -> [u8; 8] {
    [0x10, 0x02, 0x02, left_speed, right_speed, left_speed ^ right_speed, 0x10, 0x03]
}

pub async fn websocket_connection(ws: WebSocket, shared_state: Arc<Mutex<State>>) {
    let (_client_ws_sender, mut client_ws_rcv) = ws.split();

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