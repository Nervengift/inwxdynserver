mod inwx;

extern crate reqwest;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use std::fmt::Display;

const DOMAIN_ID: u32 = 123456789;
const INWX_USER: &str = "user";
const INWX_PASS: &str = "password";
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
    match inwx::update_dns(INWX_USER, INWX_PASS, DOMAIN_ID, "2000::1:2:3") {
        Ok(_) => {},
        Err(err) => println!("Error! {}", err),
    };
}
