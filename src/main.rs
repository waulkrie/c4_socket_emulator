
use clap::{arg, command};
use std::io::Write;
use std::net::TcpStream;
use serde_json::{json, Value};

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

    /// Percent value to move motor
    #[arg(short, long)]
    percent: Option<i8>,

    /// ip address:port string for socket target
    #[arg(short, long, default_value= "172.16.10.67:5001")]
    ip: String,
}


// https://github.com/tui-rs-revival/ratatui
// #[cfg(feature = "cargo")]
fn main() {
    let ip = "172.16.10.67:5001";
    let args = Args::parse();
    println!("{args:#?}");

    if args.ip.is_empty() {
        let ip = "172.16.10.67:5001";
    } else{
        let ip = args.ip;
    }
    if args.open {
        stream_to_ip(&ip, json!({"cmd": "open"})).unwrap();
    } else if args.close {
        stream_to_ip(&ip, json!({"cmd": "close"})).unwrap();
    } else if let Some(percent) = args.percent {
        stream_to_ip(&ip, json!({"cmd": "percent", "value": percent})).unwrap();
    }

}


fn stream_to_ip(ip: &str, cmd: Value) -> Result<String, String> {
    if let Ok(mut stream) = TcpStream::connect("172.16.10.67:5001") {
        if let Ok(_ret) = stream.write(cmd.to_string().as_ref()) {
            drop(stream);
            println!("Ok ");
            return Ok("done".to_string());
        }
    }
    Err("failed to connect socket".to_string())

}
