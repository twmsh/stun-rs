use bytes::Bytes;
use log::{debug, error};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use stun_rs::attrs::address_attr::AddressAttr;
use stun_rs::attrs::change_request::ChangeRequest;
use stun_rs::attrs::errcode_attr::ErrcodeAttr;
use stun_rs::attrs::response_port::ResponsePort;
use stun_rs::attrs::xor_address::XorMappedAddress;
use stun_rs::constants::*;
use tokio::net::UdpSocket;

use stun_rs::header::Header;
use stun_rs::packet::Packet;
use stun_rs::util::print_bytes;

pub fn parse_request(buf: Bytes) -> Result<Packet, String> {
    Packet::unpack(buf).map_err(|x| format!("{:?}", x))
}

pub fn validate_req(req: &Packet) -> Option<String> {
    if req.header.msg_type != MESSAGE_TYPE_BIND_REQ {
        return Some(format!("bad request msg_type: {}", req.header.msg_type));
    }

    if let Some(e) = req.validate() {
        return Some(format!("validate fail, {:?}", e));
    }

    // check response-port && padding
    let mut has_res_port = false;
    let mut has_padding = false;

    for v in req.attrs.iter() {
        if v.attr_type == ATTR_RESPONSE_PORT {
            has_res_port = true;
        }
        if v.attr_type == ATTR_PADDING {
            has_padding = true;
        }
    }

    if has_res_port && has_padding {
        return Some("RESPONSE_PORT and PADDING appear at the same time".to_string());
    }

    None
}

// 返回响应包，响应包从哪个地址发出，发到哪个目的地址

pub fn get_response(
    req: &Packet,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    ips: [IpAddr; 2],
    ports: [u16; 2],
) -> (Packet, SocketAddr, SocketAddr) {
    let trans_id = req.header.trans_id;
    let header = Header::new(MESSAGE_TYPE_BIND_RES, 0, trans_id);

    let mapped_address_attr = AddressAttr::new(ATTR_MAPPED_ADDRESS, remote_addr);
    let response_origin_attr = AddressAttr::new(ATTR_RESPONSE_ORIGIN, local_addr);
    let source_address_attr = AddressAttr::new(ATTR_SOURCE_ADDRESS, local_addr);
    let xor_mapped_attr = XorMappedAddress::new(trans_id, remote_addr);

    let da = local_addr.ip();
    let dp = local_addr.port();
    let (ca, cp) = get_ca_cp(da, dp, ips, ports);

    let other_address_attr = AddressAttr::new(ATTR_OTHER_ADDRESS, SocketAddr::new(ca, cp));

    let changed_address_attr = AddressAttr::new(ATTR_CHANGED_ADDRESS, SocketAddr::new(ca, cp));

    let attrs = vec![
        mapped_address_attr.into(),
        response_origin_attr.into(),
        source_address_attr.into(),
        xor_mapped_attr.into(),
        other_address_attr.into(),
        changed_address_attr.into(),
    ];

    let response = Packet::new(header, attrs);

    // 检查 change-request / response-port

    let (change_ip, change_port, response_port) = get_change_flag(req);

    let dst_addr = match response_port {
        None => remote_addr,
        Some(v) => SocketAddr::new(remote_addr.ip(), v),
    };

    let src_ip = match change_ip {
        true => ca,
        false => da,
    };
    let src_port = match change_port {
        true => cp,
        false => dp,
    };

    let src_addr = SocketAddr::new(src_ip, src_port);

    (response, src_addr, dst_addr)
}

pub fn get_change_flag(req: &Packet) -> (bool, bool, Option<u16>) {
    let mut change_ip = false;
    let mut change_port = false;
    let mut response_port = None;

    for attr in req.attrs.iter() {
        if attr.attr_type == ATTR_CHANGE_REQUEST {
            let change_attr: Result<ChangeRequest, _> = attr.clone().try_into();
            if let Ok(v) = change_attr {
                change_ip = v.change_ip;
                change_port = v.change_port;
            }
        }

        if attr.attr_type == ATTR_RESPONSE_PORT {
            let change_attr: Result<ResponsePort, _> = attr.clone().try_into();
            if let Ok(v) = change_attr {
                response_port = Some(v.port);
            }
        }
    }

    (change_ip, change_port, response_port)
}

pub fn get_ca_cp(da: IpAddr, dp: u16, ips: [IpAddr; 2], ports: [u16; 2]) -> (IpAddr, u16) {
    let ca = match da == ips[0] {
        true => ips[1],
        false => ips[0],
    };

    let cp = match dp == ports[0] {
        true => ports[1],
        false => ports[0],
    };

    (ca, cp)
}

pub fn get_bad_response(
    req: &Packet,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
) -> (Packet, SocketAddr, SocketAddr) {
    let trans_id = req.header.trans_id;
    let header = Header::new(MESSAGE_TYPE_BIND_ERR_RES, 0, trans_id);

    let mut res = Packet::new(header, vec![]);
    res.add_attr(ErrcodeAttr::new(ERROR_CODE_BAD_REQUEST, "bad request").into());

    (res, local_addr, remote_addr)
}

pub async fn send_response(
    res: &Packet,
    src_addr: SocketAddr,
    dst_addr: SocketAddr,
    sockets: &HashMap<SocketAddr, Arc<UdpSocket>>,
) {
    let socket = match sockets.get(&src_addr) {
        None => {
            error!("can't find UdpSocket: {}", src_addr);
            return;
        }
        Some(v) => v.clone(),
    };

    let data = res.pack();
    match socket.send_to(&data, dst_addr).await {
        Ok(v) => {
            debug!(
                "{} ---> {}\n{}",
                src_addr,
                dst_addr,
                print_bytes(&data, " ", 8)
            );
            debug!("sent: {}", v);
        }
        Err(e) => {
            error!("error, {} ---> {}, {:?}", src_addr, dst_addr, e);
        }
    };
}
