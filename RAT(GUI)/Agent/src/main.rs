mod network;
use network::protocol::{port_scan, multi_port_scan};
use std::error::Error;
use std::io::{stdin, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ip = "127.0.0.1";
    let port_range = "80-3307";

    let _ = multi_port_scan(ip,port_range).await;
    Ok(())
}