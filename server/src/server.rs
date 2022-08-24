/*
绑定4个 socket
一个mpsc 收集数据, 记录从哪个socket发出来，源地址多少，数据buf=64k
处理完成后，选择用哪个socket发出
一个退出watch
*/

use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::watch::Receiver as WatchReceiver;

use bytes::Bytes;
use log::{debug, error};
use tokio::sync::mpsc::error::SendError;

// local addr, remote add, recv data
type SocketInput = (SocketAddr, SocketAddr, Bytes);

pub struct Server {
    ip1: IpAddr,
    ip2: IpAddr,
    port1: u16,
    port2: u16,
    signal_rx: WatchReceiver<u8>,
    queue_tx: Arc<Sender<SocketInput>>,
    queue_rx: Arc<Receiver<SocketInput>>,
    sockets: Arc<HashMap<SocketAddr, UdpSocket>>,
}

impl Server {
    pub async fn new(
        ip1: IpAddr,
        ip2: IpAddr,
        port1: u16,
        port2: u16,
        signal_rx: WatchReceiver<u8>,
    ) -> io::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel::<SocketInput>(100);
        let map = init_socket(ip1, ip2, port1, port2).await?;

        let server = Self {
            ip1,
            ip2,
            port1,
            port2,
            signal_rx,
            queue_tx: Arc::new(queue_tx),
            queue_rx: Arc::new(queue_rx),
            sockets: Arc::new(map),
        };
        Ok(server)
    }

    pub async fn run(self) {}
}

async fn init_socket(
    ip1: IpAddr,
    ip2: IpAddr,
    port1: u16,
    port2: u16,
) -> io::Result<HashMap<SocketAddr, UdpSocket>> {
    let mut sockets = HashMap::with_capacity(4);

    // bind, 互为 CA, CP
    for ip in [ip1, ip2] {
        for port in [port1, port2] {
            let pair = SocketAddr::new(ip, port);
            let socket = UdpSocket::bind(pair).await?;
            sockets.insert(pair, socket);
        }
    }

    Ok(sockets)
}

async fn recv_udp(
    socket: Arc<UdpSocket>,
    local_addr: SocketAddr,
    sender: Arc<Sender<SocketInput>>,
    mut signal_rx: WatchReceiver<u8>,
) {
    let mut buf = vec![0u8; 32 * 1024];

    loop {
        tokio::select! {
            Ok((len,remote_addr)) = socket.recv_from(&mut buf) => {
                let data = Bytes::copy_from_slice(&buf[..len]);
                match sender.send((local_addr,remote_addr,data)).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("error, recv_udp, {}, {:?}",local_addr,e);
                    }
                };
            },
             _ = signal_rx.changed() => {
                debug!("recv signal, recv_udp, {} will exit.", local_addr);
                break;
            }
        }
    }
}
