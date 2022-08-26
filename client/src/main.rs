use std::net::{IpAddr, SocketAddr};

use clap::builder::ValueParser;
use clap::{Arg, Command};
use log::debug;
use client::client::probe_nat;
use tokio::net::UdpSocket;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn parse_ip(s: &str) -> Result<IpAddr, String> {
    let ip = match s.parse::<IpAddr>() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{}", e));
        }
    };
    // 不能是 0.0.0.0
    match ip {
        IpAddr::V4(ip) => {
            let value = u32::from_be_bytes(ip.octets());
            if value == 0 {
                return Err("0.0.0.0 not allow".to_string());
            }
        }
        IpAddr::V6(_) => {
            return Err("ipv6 not support".to_string());
        }
    }

    Ok(ip)
}

fn parse_addr(s: &str) -> Result<SocketAddr, String> {
    let addr = match s.parse::<SocketAddr>() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{}", e));
        }
    };
    // 不能是 0.0.0.0
    match addr {
        SocketAddr::V4(addr_v4) => {
            let value = u32::from_be_bytes(addr_v4.ip().octets());
            if value == 0 {
                return Err("0.0.0.0 not allow".to_string());
            }
        }
        SocketAddr::V6(_) => {
            return Err("ipv6 not support".to_string());
        }
    }

    Ok(addr)
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Command::new(APP_NAME)
        .version(APP_VERSION)
        .about("a stun client for probing nat")
        .arg(
            Arg::new("server")
                .long("server")
                .takes_value(true)
                .required(true)
                .help("server address")
                .value_parser(ValueParser::new(parse_addr)),
        )
        .arg(
            Arg::new("local_ip")
                .long("local_ip")
                .takes_value(true)
                .required(true)
                .help("local ip")
                .value_parser(ValueParser::new(parse_ip)),
        )
        .get_matches();

    let server: SocketAddr = *app.get_one("server").expect("wrong server address");
    let local_ip: IpAddr = *app.get_one("local_ip").expect("wrong local ip");

    let sock = UdpSocket::bind(format!("{}:0", local_ip))
        .await
        .expect("can't bind");

    let local_addr = sock.local_addr();
    debug!("local addr: {:?}", local_addr);

    probe_nat(&sock, server).await;
}
