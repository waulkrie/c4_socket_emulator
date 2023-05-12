use std::{io, thread};
use std::error::Error;
use clap::{arg, command};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use serde_json::{json};
use clap::Parser;

/// Simple program to emulate control4
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Open Cmd
    #[arg(short, long)]
    open: bool,

    /// Close cmd
    #[arg(short, long)]
    close: bool,

    /// stress test with stats
    #[arg(short, long)]
    stress: bool,

    /// Percent value to move motor
    #[arg(short, long)]
    percent: Option<i8>,

    /// ip address:port string for socket target
    #[arg(short, long, default_value= "172.16.10.67:5001")]
    ip: String,
}


// https://github.com/tui-rs-revival/ratatui
fn main() {
    let mut ip:&str = "172.16.10.67:5001";
    let args:Args = Args::parse();
    // println!("{args:#?}");

    if !args.ip.is_empty() {
        ip = args.ip.as_str();
    }
    println!("ip: {:?}", ip);
    if args.stress {
        stress_test(ip);
    } else {
        let json:String = convert_args_json(&args);
        stream_to_ip(&ip, &json).unwrap();
    }

}

fn stress_test(ip: &str){
    let mut i:u64 = 0;
    let mut read_failures:u32 = 0;
    let mut write_failures:u32 = 0;
    let mut conn_refused:u32 = 0;
    let mut timed_out:u32 = 0;
    let mut success:u32 = 0;

    let mut avg_resp_ms:u128 = 0;
    let mut running_total:u128 = 0;
    let mut min_resp_ms:u128 = u128::MAX;
    let mut max_resp_ms:u128 = u128::MIN;

    loop {
        match stream_to_ip_with_response(ip, generate_json_cmd(i).as_str()) {
            Ok(resp) => {
                success += 1;
                running_total += resp.as_millis();
                avg_resp_ms = running_total/(success as u128);
                if min_resp_ms > resp.as_millis() {
                    min_resp_ms = resp.as_millis();
                }
                if max_resp_ms < resp.as_millis() {
                    max_resp_ms = resp.as_millis();
                }
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::WriteZero => {
                        write_failures += 1;
                        println!("failure to write: {}", write_failures);
                        continue;
                    },
                    ErrorKind::WouldBlock => {
                        read_failures += 1;
                        println!("timeout, failure to read: {}", read_failures);
                        continue;
                    },
                    ErrorKind::ConnectionRefused => {
                        conn_refused += 1;
                        println!("ConnectionRefused: {}", conn_refused);
                        continue;
                    },
                    ErrorKind::TimedOut => {
                        timed_out += 1;
                        println!("TimedOut:  {}", timed_out);
                        println!("TimedOut...{}", err.to_string());
                        continue;
                    },
                    _=> {
                        println!("OTHER ERROR: {:?}", err.kind());
                        break;
                    }

                }
            }
        }
        i += 1;
        thread::sleep(Duration::from_secs(1));
        if i % 100 == 0 {
            // print stats
            println!("write_failures {}", write_failures);
            println!("read  timeouts {}", read_failures);
            println!("ConnRefused    {}", conn_refused);
            println!("timed_out      {}", timed_out);
            println!("# success      {}", success);
            println!("avg_resp_ms    {}", avg_resp_ms);
            println!("min_resp_ms    {}", min_resp_ms);
            println!("max_resp_ms    {}", max_resp_ms);
        }
    }
}

fn stream_to_ip_with_response(ip: &str, cmd: &str) -> io::Result<Duration> {
    let send_cmd = cmd.to_string();

    let mut stream = TcpStream::connect(ip)?;

    let timeout = Option::from(Duration::new(5, 0));
    stream.set_read_timeout(timeout).expect("Could not set socket read timeout");

    let start_time = Instant::now();
    stream.write(send_cmd.as_ref())?;
    let mut receive_buffer: [u8; 1000] = [0; 1000];
    let bytes_read = stream.read(&mut receive_buffer)?;
    let response_time = start_time.elapsed();

    let received_cmd = std::str::from_utf8(&receive_buffer[0..bytes_read])
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
        .to_string();

    if received_cmd == send_cmd {
        Ok(response_time)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Sent and received commands are not equal",
        ))
    }
}

//
// fn stream_to_ip_with_response(ip: &str, cmd: &str) -> io::Result<()> {
//     let mut recv_cmd:String = String::new();
//     let send_cmd:String = cmd.to_string();
//
//     let mut stream = TcpStream::connect(ip)?;
//     let timeout = Option::from(Duration::new(5, 0));
//     stream.set_read_timeout(timeout)
//         .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
//
//     let mut receive_buffer: [u8; 1000] = [0; 1000];
//     stream.write(send_cmd.as_bytes())
//         .map_err(|err| io::Error::new(io::ErrorKind::WriteZero, err))?;
//     // println!("SENT {}", send_cmd);
//
//     loop {
//         let bytes_read = stream.read(&mut receive_buffer)?;
//         if bytes_read == 0 {
//             break;
//         }
//         let chunk = std::str::from_utf8(&receive_buffer[0..bytes_read])
//             .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
//
//         recv_cmd.push_str(chunk);
//         if recv_cmd == send_cmd {
//             return Ok(());
//         }
//     }
//
//     Err(io::Error::new(io::ErrorKind::Other, "Sent and received commands are not equal"))
//
// }


fn stream_to_ip(ip: &str, cmd: &str) -> Result<String, String> {
    if let Ok(mut stream) = TcpStream::connect(ip) {
        if let Ok(_ret) = stream.write(cmd.to_string().as_ref()) {
            // stream.read(&mut recieve_buffer).expect("TODO: panic message");
            drop(stream);
            return Ok("Ok".to_string());
        }
    }
    Err("failed to connect socket".to_string())
}



fn convert_args_json(args: &Args) -> String {
    if args.open {
        json!({"cmd": "open"}).to_string()
    } else if args.close {
        json!({"cmd": "close"}).to_string()
    } else if let Some(percent) = args.percent {
        json!({"cmd": "percent", "value": percent}).to_string()
    } else {
        json!({"cmd": "open"}).to_string()
    }
}

fn generate_json_cmd(i:u64) -> String {
    match i {
        n if n % 10 == 0 => json!({"cmd": "percent", "value": n % 100}).to_string(),
        n if n % 2 == 0 => json!({"cmd": "open"}).to_string(),
        n if n % 2 == 1 => json!({"cmd": "close"}).to_string(),
        n => {
            println!("{} is none of the above", n);
            json!({"cmd": "open"}).to_string()
        },
    } // end match
}