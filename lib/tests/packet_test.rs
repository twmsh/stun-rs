#![allow(clippy::vec_init_then_push)]

use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use stun_rs::attrs::address_attr::AddressAttr;
use stun_rs::attrs::change_request::ChangeRequest;
use stun_rs::attrs::errcode_attr::ErrcodeAttr;
use stun_rs::attrs::response_port::ResponsePort;
use stun_rs::attrs::xor_address::XorMappedAddress;

use stun_rs::constants::*;
use stun_rs::header::Header;
use stun_rs::packet::Packet;
use stun_rs::util;

#[test]
pub fn test_print_ipv6() {
    let addr = SocketAddrV6::new(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 8080, 0, 0);
    println!("{:?}", addr);
}

#[test]
pub fn test_new_trans_id() {
    let trans_id = util::new_trans_id();
    println!("{}", util::print_bytes(&trans_id, " ", 8));
}

#[test]
pub fn test_new_err_response_packet() {
    let trans_id = util::new_trans_id();

    let header = Header::new(MESSAGE_TYPE_BIND_ERR_RES, 0, trans_id);
    let mut attr_list = Vec::new();
    attr_list.push(ErrcodeAttr::new(502, "not auth").into());

    let packet = Packet::new(header, attr_list);
    let buf = packet.pack();

    println!("{}", util::print_bytes(&buf, " ", 8));
}

#[test]
pub fn test_new_response_packet() {
    let trans_id = util::new_trans_id();

    let header = Header::new(MESSAGE_TYPE_BIND_RES, 0, trans_id);
    let mut attr_list = Vec::new();

    // let mapped_addr:SocketAddr = "192.168.8.100:5678".parse().expect("unable to parse");
    let mapped_addr: SocketAddr = "[1:2:3:4:5:6:7:8]:8080".parse().expect("unable to parse");
    let origin_addr: SocketAddr = "10.20.30.40:1234".parse().expect("unable to parse");
    let other_addr: SocketAddr = "10.20.30.41:1235".parse().expect("unable to parse");

    attr_list.push(AddressAttr::new(ATTR_MAPPED_ADDRESS, mapped_addr).into());
    attr_list.push(AddressAttr::new(ATTR_RESPONSE_ORIGIN, origin_addr).into());
    attr_list.push(AddressAttr::new(ATTR_OTHER_ADDRESS, other_addr).into());
    attr_list.push(XorMappedAddress::new(trans_id, mapped_addr).into());

    let packet = Packet::new(header, attr_list);
    let buf = packet.pack();

    println!("{}", util::print_bytes(&buf, " ", 8));
}

#[test]
pub fn test_new_req_packet() {
    let trans_id = util::new_trans_id();

    let header = Header::new(MESSAGE_TYPE_BIND_REQ, 0, trans_id);
    let mut attr_list = Vec::new();

    attr_list.push(ChangeRequest::new(false, true).into());
    attr_list.push(ResponsePort::new(8080).into());
    attr_list.push(util::new_padding_attr(16).into());

    let packet = Packet::new(header, attr_list);
    let buf = packet.pack();

    println!("{}", util::print_bytes(&buf, " ", 8));
}

#[test]
pub fn test_unpack_req() {
    let trans_id = util::new_trans_id();

    let header = Header::new(MESSAGE_TYPE_BIND_REQ, 0, trans_id);
    let mut attr_list = Vec::new();

    attr_list.push(ChangeRequest::new(false, true).into());
    attr_list.push(ResponsePort::new(8080).into());
    attr_list.push(util::new_padding_attr(16).into());

    let packet = Packet::new(header, attr_list);
    let buf = packet.pack();

    println!("{}", util::print_bytes(&buf, " ", 8));
    println!("----------------------");
    let packet = Packet::unpack(buf).unwrap();
    println!("{:?}", packet);
}

#[test]
pub fn test_unpack_response() {
    let trans_id = util::new_trans_id();

    let header = Header::new(MESSAGE_TYPE_BIND_RES, 0, trans_id);
    let mut attr_list = Vec::new();

    // let mapped_addr:SocketAddr = "192.168.8.100:5678".parse().expect("unable to parse");
    let mapped_addr: SocketAddr = "[1:2:3:4:5:6:7:8]:8080".parse().expect("unable to parse");
    let origin_addr: SocketAddr = "10.20.30.40:1234".parse().expect("unable to parse");
    let other_addr: SocketAddr = "10.20.30.41:1235".parse().expect("unable to parse");

    attr_list.push(AddressAttr::new(ATTR_MAPPED_ADDRESS, mapped_addr).into());
    attr_list.push(AddressAttr::new(ATTR_RESPONSE_ORIGIN, origin_addr).into());
    attr_list.push(AddressAttr::new(ATTR_OTHER_ADDRESS, other_addr).into());
    attr_list.push(XorMappedAddress::new(trans_id, mapped_addr).into());

    let packet = Packet::new(header, attr_list);
    let buf = packet.pack();

    println!("{}", util::print_bytes(&buf, " ", 8));
    println!("----------------------");
    let packet = Packet::unpack(buf).unwrap();
    println!("{:?}", packet);
}
