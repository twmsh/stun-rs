// ./server --ip1 1.2.3.4 --ip2 1.2.3.5 --port1 3478 --port2 3479

use log::{debug, error, info};
use std::net::IpAddr;

use clap::builder::ValueParser;
use clap::{Arg, Command};
use tokio::sync::watch;

use server::server::Server;
use server::signal::wait_shutdown;

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

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Command::new(APP_NAME)
        .version(APP_VERSION)
        .about("a small stun server")
        .arg(
            Arg::new("ip1")
                .long("ip1")
                .takes_value(true)
                .required(true)
                .help("primary ip")
                .value_parser(ValueParser::new(parse_ip)),
        )
        .arg(
            Arg::new("ip2")
                .long("ip2")
                .takes_value(true)
                .required(true)
                .help("alternative ip")
                .value_parser(ValueParser::new(parse_ip)),
        )
        .arg(
            Arg::new("port1")
                .long("port1")
                .takes_value(true)
                .required(true)
                .help("primary port")
                .value_parser(clap::value_parser!(u16).range(0..65535)),
        )
        .arg(
            Arg::new("port2")
                .long("port2")
                .takes_value(true)
                .required(true)
                .help("alternative port")
                .value_parser(clap::value_parser!(u16).range(0..65535)),
        )
        .get_matches();

    //
    let ip1: IpAddr = *app.get_one("ip1").expect("wrong ip1");
    let ip2: IpAddr = *app.get_one("ip2").expect("wrong ip2");

    let port1: u16 = *app.get_one("port1").expect("wrong port1");
    let port2: u16 = *app.get_one("port2").expect("wrong port2");

    if ip1 == ip2 {
        panic!("error, ip1 equal ip2");
    }
    if port1 == port2 {
        panic!("error, port1 equal port2");
    }

    debug!("ip:{},{}  port:{},{}", ip1, ip2, port1, port2);

    let (signal_tx, signal_rx) = watch::channel(0_u8);

    let _signal_handle = tokio::spawn(async move {
        wait_shutdown().await;
        match signal_tx.send(1) {
            Ok(_) => {}
            Err(e) => {
                error!("error, {:?}", e);
            }
        };
    });

    let server = match Server::new([ip1, ip2], [port1, port2], signal_rx).await {
        Ok(v) => v,
        Err(e) => {
            panic!("error, {:?}", e);
        }
    };

    let server_handle = tokio::spawn(async move {
        server.run().await;
    });

    info!("start server ...");

    let _ = server_handle.await;

    println!("end.");
}
