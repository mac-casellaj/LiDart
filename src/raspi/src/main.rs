use std::time::Instant;
use serde::Serialize;
use warp::{reply::json, Reply, Filter, ws::WebSocket};
use futures::StreamExt;
use std::time::Duration;
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits, available_ports, SerialPortType};
use clap::{Arg, Command};

#[tokio::main]
async fn main() {

    let start_time = Instant::now();
    let health_route = warp::path("health").and_then(move || health_handler(start_time));

    // Open the serial port
    //Defininf the serial port for communication
     let port = serialport::new("/dev/ttyUSB0", 115_200)
     .timeout(Duration::from_millis(10))
     .open().expect("Failed to open port");
     //println!("Port name{}", port);

     //Remove this in order to test the functionality of the sending function
     match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                println!("  {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("    Type: USB");
                        println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        println!(
                            "     Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Type: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }

    //Sending data to serialport
    //This is to be modified with the top available lists function 

    // let matches = Command::new("Serialport example communnication")
    //     .about("Write bytes to a serial port at 1Hz")
    //     .disable_version_flag(true)
    //     .arg(
    //         Arg::new("port")
    //             .help("The device path to a serial port")
    //             .required(true),
    //     )
    //     .arg(
    //         Arg::new("baud")
    //             .help("The baud rate to connect at")
    //             .use_value_delimiter(false)
    //             .required(true)
    //             .validator(valid_baud),
    //     )
    //     .arg(
    //         Arg::new("stop-bits")
    //             .long("stop-bits")
    //             .help("Number of stop bits to use")
    //             .takes_value(true)
    //             .possible_values(&["1", "2"])
    //             .default_value("1"),
    //     )
    //     .arg(
    //         Arg::new("data-bits")
    //             .long("data-bits")
    //             .help("Number of data bits to use")
    //             .takes_value(true)
    //             .possible_values(&["5", "6", "7", "8"])
    //             .default_value("8"),
    //     )
    //     .arg(
    //         Arg::new("rate")
    //             .long("rate")
    //             .help("Frequency (Hz) to repeat transmission of the pattern (0 indicates sending only once")
    //             .takes_value(true)
    //             .default_value("1"),
    //     )
    //     .arg(
    //         Arg::new("string")
    //             .long("string")
    //             .help("String to transmit")
    //             .takes_value(true)
    //             .default_value("."),
    //     )
    //     .get_matches();

    // let port_name = matches.value_of("port").unwrap();
    // let baud_rate = matches.value_of("baud").unwrap().parse::<u32>().unwrap();
    // let stop_bits = match matches.value_of("stop-bits") {
    //     Some("2") => StopBits::Two,
    //     _ => StopBits::One,
    // };
    // let data_bits = match matches.value_of("data-bits") {
    //     Some("5") => DataBits::Five,
    //     Some("6") => DataBits::Six,
    //     Some("7") => DataBits::Seven,
    //     _ => DataBits::Eight,
    // };
    // let rate = matches.value_of("rate").unwrap().parse::<u32>().unwrap();
    // let string = matches.value_of("string").unwrap();

    // let builder = serialport::new(port_name, baud_rate)
    //     .stop_bits(stop_bits)
    //     .data_bits(data_bits);
    // println!("{:?}", &builder);
    // let mut port = builder.open().unwrap_or_else(|e| {
    //     eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
    //     ::std::process::exit(1);
    // });

    // println!(
    //     "Writing '{}' to {} at {} baud at {}Hz",
    //     &string, &port_name, &baud_rate, &rate
    // );
    // loop {
    //     match port.write(string.as_bytes()) {
    //         Ok(_) => {
    //             print!("{}", &string);
    //             std::io::stdout().flush().unwrap();
    //         }
    //         Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
    //         Err(e) => eprintln!("{:?}", e),
    //     }
    //     if rate == 0 {
    //         return;
    //     }
    //     std::thread::sleep(Duration::from_millis((1000.0 / (rate as f32)) as u64));
    // }

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

    macro_rules! call_query_method_check {
        ($port:ident, $func:path) => {
            match $func($port) {
                Ok(_) => println!("  {}: success", stringify!($func)),
                Err(ref e) => println!("  {}: FAILED ({})", stringify!($func), e),
            }
        };
    }
    
    
    fn test_single_port(port: &mut dyn serialport::SerialPort, loopback: bool) 
    {
        println!("Testing '{}':", port.name().unwrap());
        println!("Testing bytes to read and write...");
        call_query_method_check!(port, SerialPort::bytes_to_write);
        call_query_method_check!(port, SerialPort::bytes_to_read);
    }
}

pub async fn ws_handler(ws: warp::ws::Ws) -> Result<impl Reply, warp::Rejection> {
    Ok(ws.on_upgrade(websocket_connection))
}

pub async fn websocket_connection(ws: WebSocket) {
    let (mut _client_ws_sender, mut client_ws_rcv) = ws.split();

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

        println!("Got {}", msg_text); //From the client
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

