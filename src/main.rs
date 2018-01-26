mod inwx;
mod dns;

extern crate reqwest;
extern crate trust_dns;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

extern crate hyper;
extern crate futures;

use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::collections::HashMap;
use std::sync::Arc;

use std::str::FromStr;
use trust_dns::rr::Name;

use futures::Future;
use futures::Stream;
use hyper::server::{Http, Request, Response, Service};
use hyper::header::ContentLength;
use hyper::{Method, StatusCode};


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

type TokenMap = HashMap<String, (Name, u32)>;

struct UpdateService {
    tokens: Arc<TokenMap>,
}

impl Service for UpdateService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match (req.method(), req.path()) {
            (&Method::Get, "/update") => {
                Box::new(futures::future::ok(Response::new()
                                             .with_status(StatusCode::MethodNotAllowed)
                                             .with_body("try POSTind data")))
            },
            (&Method::Post, "/update") => {
                let new_addr = req.remote_addr().unwrap().ip();  // TODO: when will this fail?
                let tokens = self.tokens.clone();
                Box::new(req.body().concat2().map(move |b| {
                    let token = String::from_utf8_lossy(b.as_ref().into());
                    println!("{}", token);
                    let (hostname, domain_id) = match tokens.get(&*token) {
                        Some(&(ref hostname, domain_id)) => (hostname, domain_id),
                        None => {
                            return Response::new()
                                .with_status(StatusCode::Unauthorized)
                                .with_body("unknown token")},
                    };

                    let changed = match new_addr {
                        IpAddr::V4(new_ip) => {
                            let old_ip = dns::lookup::<Ipv4Addr>(Name::from_str(HOSTNAME).unwrap());
                            match old_ip {
                                Ok(Some(ip)) => new_ip != ip,
                                // TODO: log errors?
                                _ => true,
                            }
                        },
                        IpAddr::V6(new_ip) => {
                            let old_ip = dns::lookup::<Ipv6Addr>(Name::from_str(HOSTNAME).unwrap());
                            match old_ip {
                                Ok(Some(ip)) => new_ip != ip,
                                // TODO: log errors?
                                _ => true,
                            }
                        },
                    };

                    if changed {
                        match inwx::update_dns(INWX_USER, INWX_PASS, domain_id, new_addr) {
                            Ok(_) => println!("Changed ip to {}", new_addr),
                            Err(err) => return Response::new()
                                .with_status(StatusCode::InternalServerError)
                                .with_body("NACK could not update DNS record"),
                        }
                    } else {
                        println!("No change");
                    }

                    let body = "ACK";
                    Response::new()
                        .with_header(ContentLength(body.len() as u64))
                        .with_body(body)
                }))
            },
            (_, "/") => {
                Box::new(futures::future::ok(Response::new()
                                             .with_status(StatusCode::NotFound)
                                             .with_body("currently the only endpoint is /update")))
            }
            _ => {
                Box::new(futures::future::ok(Response::new()
                                             .with_status(StatusCode::NotFound)))
            },
        }
    }
}

fn main() {
    // access token -> domain-id mapping
    let mut tokens = TokenMap::new();
    tokens.insert("asdf".to_owned(), (Name::from_str("obstsalat.nerven.gift").unwrap(), 259266683));
    println!("{:?}", tokens);
    let token_pt = Arc::new(tokens);

    let addr = "[::]:3000".parse().unwrap();
    let server = Http::new().bind(&addr, move || Ok(UpdateService{tokens:token_pt.clone()})).unwrap();
    server.run().unwrap();
}
