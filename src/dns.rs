extern crate trust_dns;

use trust_dns::client::{Client, ClientConnection, ClientStreamHandle, SyncClient};
use trust_dns::udp::UdpClientConnection;
use std::str::FromStr;
use trust_dns::op::Message;
use trust_dns::rr::{DNSClass, Name, RData, Record, RecordType};

use std::net::Ipv6Addr;

const DNS_SERVER: &str = "192.174.68.104:53";

pub fn lookup_v6(hostname: Name) -> Result<Ipv6Addr, trust_dns::error::ClientError> {
    let address = DNS_SERVER.parse().unwrap();
    let conn = UdpClientConnection::new(address)?;
    let client = SyncClient::new(conn);
    let response: Message = client.query(&hostname, DNSClass::IN, RecordType::AAAA)?;
    let answers: &[Record] = response.answers();
    if let &RData::AAAA(ref ip) = answers[0].rdata() {
        return Ok(*ip);
    } else {
        // TODO: error handling
        panic!("strange message received!");
    }
}
