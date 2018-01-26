extern crate trust_dns;

use trust_dns::client::{Client, ClientConnection, ClientStreamHandle, SyncClient};
use trust_dns::udp::UdpClientConnection;
use std::str::FromStr;
use trust_dns::op::Message;
use trust_dns::rr::{DNSClass, Name, RData, Record, RecordType};

use std::net::{Ipv4Addr, Ipv6Addr};

const DNS_SERVER: &str = "192.174.68.104:53";

pub trait HasIpAddr {
    type Addr;
    fn record_type() -> RecordType;
    fn get_ip(rdata: &RData) -> Option<Self::Addr>;

}

impl HasIpAddr for Ipv4Addr {
    type Addr = Ipv4Addr;
    fn record_type() -> RecordType {
        RecordType::A
    }
    fn get_ip(rdata: &RData) -> Option<Self::Addr> {
        match rdata {
            &RData::A(ip) => Some(ip),
            _ => None,
        }
    }
}

impl HasIpAddr for Ipv6Addr {
    type Addr = Ipv6Addr;
    fn record_type() -> RecordType {
        RecordType::AAAA
    }
    fn get_ip(rdata: &RData) -> Option<Self::Addr> {
        match rdata {
            &RData::AAAA(ip) => Some(ip),
            _ => None,
        }
    }
}

pub fn lookup<T:HasIpAddr>(hostname: Name) -> Result<Option<T::Addr>, trust_dns::error::ClientError> {
    let address = DNS_SERVER.parse().unwrap();
    let conn = UdpClientConnection::new(address)?;
    let client = SyncClient::new(conn);
    let response: Message = client.query(&hostname, DNSClass::IN, T::record_type())?;
    let answers: &[Record] = response.answers();
    let ip = answers.get(0).and_then(|x| T::get_ip(x.rdata())); //TODO: multiple records?
    Ok(ip)
}
