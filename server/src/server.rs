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
use stun_rs::util::print_bytes;

use crate::stun::{get_bad_response, get_response, parse_request, send_response, validate_req};

// local addr, remote addr, recv data
type SocketInput = (SocketAddr, SocketAddr, Bytes);

pub struct Server {
    ips: [IpAddr; 2],
    ports: [u16; 2],
    signal_rx: WatchReceiver<u8>,
    queue_tx: Arc<Sender<SocketInput>>,
    queue_rx: Receiver<SocketInput>,
    sockets: HashMap<SocketAddr, Arc<UdpSocket>>,
}

impl Server {
    pub async fn new(
        ips: [IpAddr; 2],
        ports: [u16; 2],
        signal_rx: WatchReceiver<u8>,
    ) -> io::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel::<SocketInput>(100);
        let map = init_socket(ips, ports).await?;

        let server = Self {
            ips,
            ports,
            signal_rx,
            queue_tx: Arc::new(queue_tx),
            queue_rx,
            sockets: map,
        };
        Ok(server)
    }

    pub async fn run(self) {
        let mut handles = vec![];

        for (addr, udp) in self.sockets.iter() {
            let socket = udp.clone();
            let local_addr = *addr;
            let sender = self.queue_tx.clone();
            let signal_rx = self.signal_rx.clone();

            let h = tokio::spawn(async move {
                recv_udp(socket, local_addr, sender, signal_rx).await;
            });
            handles.push(h);
        }

        let h = tokio::spawn(async move {
            process_udp(
                self.queue_rx,
                self.signal_rx,
                self.ips,
                self.ports,
                self.sockets,
            )
            .await;
        });
        handles.push(h);

        for v in handles {
            let _ = v.await;
        }
    }
}

//--------------------------------------------------

async fn init_socket(
    ips: [IpAddr; 2],
    ports: [u16; 2],
) -> io::Result<HashMap<SocketAddr, Arc<UdpSocket>>> {
    let mut sockets = HashMap::with_capacity(4);

    // bind, 互为 CA, CP
    for ip in ips {
        for port in ports {
            let pair = SocketAddr::new(ip, port);
            let socket = UdpSocket::bind(pair).await?;
            debug!("listening: {:?}", socket.local_addr());
            sockets.insert(pair, Arc::new(socket));
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

                debug!("recv len: {}", data.len());
                debug!("{} <--- {}\n{}",local_addr,remote_addr,print_bytes(&data," ",8));

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

async fn process_udp(
    mut receiver: Receiver<SocketInput>,
    mut signal_rx: WatchReceiver<u8>,
    ips: [IpAddr; 2],
    ports: [u16; 2],
    sockets: HashMap<SocketAddr, Arc<UdpSocket>>,
) {
    loop {
        tokio::select! {
            Some(input) = receiver.recv() => {
               process_one(input,ips,ports,&sockets).await;
            },
             _ = signal_rx.changed() => {
                debug!("recv signal, process_input, will exit.");
                break;
            }
        }
    }
}

async fn process_one(
    input: SocketInput,
    ips: [IpAddr; 2],
    ports: [u16; 2],
    sockets: &HashMap<SocketAddr, Arc<UdpSocket>>,
) {
    // 解析请求数据包
    // 组装响应包
    // 找到对应的socket 发送

    let (local_addr, remote_addr, buf) = input;
    let request = match parse_request(buf) {
        Ok(v) => v,
        Err(e) => {
            error!(
                "parse error, from remote:{}, local:{}, {:?}",
                remote_addr, local_addr, e
            );
            return;
        }
    };

    if let Some(e) = validate_req(&request) {
        error!(
            "validate error, from remote:{}, local:{}, {}",
            remote_addr, local_addr, e
        );

        // send err response
        let (response, src_addr, dst_addr) = get_bad_response(&request, local_addr, remote_addr);
        send_response(&response, src_addr, dst_addr, sockets).await;

        return;
    }

    let (response, src_addr, dst_addr) =
        get_response(&request, local_addr, remote_addr, ips, ports);
    send_response(&response, src_addr, dst_addr, sockets).await;
}
