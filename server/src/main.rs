// ./server --ip1 1.2.3.4 --ip2 1.2.3.5 --port1 3478 --port2 3479

use clap::{Arg, Command};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");


fn main() {
    let app = Command::new(APP_NAME)
        .version(APP_VERSION)
        .about("a small stun server")
        .arg(
            Arg::new("ip1")
                .long("ip1")
                .takes_value(true)
                .required(true)
                .help("primary ip")
        ).arg(
        Arg::new("ip2")
            .long("ip2")
            .takes_value(true)
            .required(true)
            .help("alternative ip")
    ).arg(
        Arg::new("port1")
            .long("port1")
            .takes_value(true)
            .required(true)
            .help("primary port")
    ).arg(
        Arg::new("port2")
            .long("port2")
            .takes_value(true)
            .required(true)
            .help("alternative port")
    ).get_matches();


    println!("end.");
}
