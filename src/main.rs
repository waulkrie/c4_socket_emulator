
use clap::{arg, command};
use std::io::Write;
use std::net::TcpStream;
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
    let json:String = convert_args_json(&args);
    stream_to_ip(&ip, &json).unwrap();

}


fn stream_to_ip(ip: &str, cmd: &str) -> Result<String, String> {
    if let Ok(mut stream) = TcpStream::connect(ip) {
        if let Ok(_ret) = stream.write(cmd.to_string().as_ref()) {
            drop(stream);
            println!("Ok ");
            return Ok("done".to_string());
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