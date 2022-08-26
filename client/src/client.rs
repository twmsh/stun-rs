use bytes::Bytes;
use log::{debug, error};
use std::io;
use std::io::Error;
use std::net::SocketAddr;
use stun_rs::attrs::address_attr::AddressAttr;
use stun_rs::attrs::change_request::ChangeRequest;
use stun_rs::attrs::response_port::ResponsePort;
use stun_rs::attrs::xor_address::XorMappedAddress;
use stun_rs::constants::{
    ATTR_MAPPED_ADDRESS, ATTR_OTHER_ADDRESS, ATTR_RESPONSE_ORIGIN, ATTR_XOR_MAPPED_ADDRESS,
    MESSAGE_TYPE_BIND_REQ,
};
use stun_rs::error::{ParsePacketErr, ValidateErr};
use stun_rs::header::{Header, TransId};
use stun_rs::packet::Packet;
use stun_rs::util::{new_trans_id, print_bytes};
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct ProbeError(pub String);

impl From<io::Error> for ProbeError {
    fn from(e: Error) -> Self {
        ProbeError(format!("{}", e))
    }
}

impl From<ParsePacketErr> for ProbeError {
    fn from(e: ParsePacketErr) -> Self {
        ProbeError(format!("{:?}", e))
    }
}

impl From<ValidateErr> for ProbeError {
    fn from(e: ValidateErr) -> Self {
        ProbeError(e.0)
    }
}

impl From<String> for ProbeError {
    fn from(e: String) -> Self {
        ProbeError(e)
    }
}

//--------------------------------------
pub struct ResponseAddressAttr {
    pub mapped_address: SocketAddr,
    pub response_origin: SocketAddr,
    pub other_address: SocketAddr,
    pub xor_mapped_address: SocketAddr,
}

//---------------------------------------
fn new_request(
    trans_id: TransId,
    change_request: Option<(bool, bool)>,
    response_port: Option<u16>,
) -> Packet {
    let header = Header::new(MESSAGE_TYPE_BIND_REQ, 0, trans_id);
    let mut request = Packet::new(header, vec![]);

    if let Some((change_ip, change_port)) = change_request {
        let attr = ChangeRequest::new(change_ip, change_port);
        request.add_attr(attr.into());
    }

    if let Some(port) = response_port {
        let attr = ResponsePort::new(port);
        request.add_attr(attr.into());
    }

    request
}

pub async fn probe_nat(sock: &UdpSocket, server: SocketAddr) {
    match probe_nat_1(sock, server).await {
        Ok(v) => {
            debug!("mapped_address: {}", v.mapped_address);
            debug!("response_origin: {}", v.response_origin);
            debug!("other_address: {}", v.other_address);
            debug!("xor_mapped_address: {}", v.xor_mapped_address);
        }
        Err(e) => {
            error!("error, probe_nat_1, {:?}", e);
        }
    }
}

pub async fn probe_nat_1(
    sock: &UdpSocket,
    server: SocketAddr,
) -> Result<ResponseAddressAttr, ProbeError> {
    let trans_id = new_trans_id();
    let mut recv_buf = vec![0u8; 32 * 1024];

    let req = new_request(trans_id, None, None);
    let buf = req.pack();
    debug!("request len: {}", buf.len());
    debug!(
        "{:?} --> {}\n{}",
        sock.local_addr().unwrap(),
        server,
        print_bytes(&buf, " ", 8)
    );

    let sent = sock.send_to(&buf, server).await?;
    debug!("sent: {}", sent);

    let (len, remote_addr) = sock.recv_from(&mut recv_buf).await?;
    let buf = Bytes::copy_from_slice(&recv_buf[..len]);
    debug!("recv len: {}", buf.len());
    debug!(
        "{:?} <-- {}\n{}",
        sock.local_addr().unwrap(),
        remote_addr,
        print_bytes(&buf, " ", 8)
    );

    let response = Packet::unpack(buf)?;
    match response.validate() {
        None => {}
        Some(e) => {
            return Err(e.into());
        }
    };

    find_response_attrs(&response)
}

fn find_response_attrs(packet: &Packet) -> Result<ResponseAddressAttr, ProbeError> {
    let mapped_address = find_address_attr(packet, ATTR_MAPPED_ADDRESS)?;
    let response_origin = find_address_attr(packet, ATTR_RESPONSE_ORIGIN)?;
    let other_address = find_address_attr(packet, ATTR_OTHER_ADDRESS)?;
    let xor_mapped_address = find_xor_address_attr(packet)?;

    Ok(ResponseAddressAttr {
        mapped_address,
        response_origin,
        other_address,
        xor_mapped_address,
    })
}

fn find_address_attr(packet: &Packet, attr_type: u16) -> Result<SocketAddr, ProbeError> {
    for attr in packet.attrs.iter() {
        if attr.attr_type == attr_type {
            let address_attr: AddressAttr = attr.clone().try_into()?;
            return Ok(address_attr.address);
        }
    }

    Err(ProbeError(format!("can't find attr: {}", attr_type)))
}

fn find_xor_address_attr(packet: &Packet) -> Result<SocketAddr, ProbeError> {
    let trans_id = &packet.header.trans_id;

    for attr in packet.attrs.iter() {
        if attr.attr_type == ATTR_XOR_MAPPED_ADDRESS {
            let address_attr: XorMappedAddress =
                XorMappedAddress::from_base_attr(attr.clone(), trans_id)?;

            return Ok(address_attr.address);
        }
    }

    Err(ProbeError(format!(
        "can't find attr: {}",
        ATTR_XOR_MAPPED_ADDRESS
    )))
}
