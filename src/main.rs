mod inwx;
mod dns;

extern crate reqwest;
extern crate trust_dns;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use std::fmt::Display;
use std::net::Ipv6Addr;

use std::str::FromStr;
use trust_dns::rr::Name;


const DOMAIN_ID: u32 = 123456789;
const INWX_USER: &str = "user";
const INWX_PASS: &str = "password";
const HOSTNAME: &str = "foo@example.com";

enum Type {
    MX,
    A,
    AAAA,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let rep = match self {
            &Type::MX => "MX",
            &Type::A => "A",
            &Type::AAAA => "AAAA",
        };
        write!(f, "{}", rep)
    }
}
fn main() {
    let new_ip = Ipv6Addr::from_str("2000::2:3:4").unwrap();

    let current_dns_ip = match dns::lookup_v6(Name::from_str(HOSTNAME).unwrap()) {
        Ok(ip) => ip,
        Err(err) => panic!("an Error occured! {}", err),
    };

    println!("{:?} -> {:?}", current_dns_ip, new_ip);
    if current_dns_ip != new_ip {
        match inwx::update_dns(INWX_USER, INWX_PASS, DOMAIN_ID, new_ip) {
            Ok(_) => println!("Changed AAAA record to {}", new_ip),
            Err(err) => panic!("Error! {}", err),
        };
    }
}
