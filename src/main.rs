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
use std::net::Ipv6Addr;
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

struct UpdateService {
    tokens: Arc<HashMap<String, u32>>,
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
                let tokens = self.tokens.clone();
                Box::new(req.body().concat2().map(move |b| {
                    let token = String::from_utf8_lossy(b.as_ref().into());
                    println!("{}", token);
                    println!("{:?}", tokens.get(&*token));
                    let body = "ok";
                    Response::new()
                        .with_header(ContentLength(body.len() as u64))
                        .with_body(body)
                }))
            },
            (_, "/") => {
                Box::new(futures::future::ok(Response::new().with_status(StatusCode::NotFound).with_body("currently only endpoint is /update")))
            }
            _ => {
                Box::new(futures::future::ok(Response::new().with_status(StatusCode::NotFound)))
            },
        }
    }
}

fn main() {
    // access token -> domain-id mapping
    let mut tokens = HashMap::new();
    tokens.insert("asdf".to_owned(), 259266683);
    println!("{:?}", tokens);
    let token_pt = Arc::new(tokens);

    let addr = "[::]:3000".parse().unwrap();
    let server = Http::new().bind(&addr, move || Ok(UpdateService{tokens:token_pt.clone()})).unwrap();
    server.run().unwrap();
    
    let new_ip = Ipv6Addr::from_str("2000::2:3:4").unwrap();

    let current_dns_ip = match dns::lookup::<Ipv6Addr>(Name::from_str(HOSTNAME).unwrap()) {
        Ok(maybe_ip) => match maybe_ip {
            Some(ip) => ip,
            None => panic!("there is no ip")
        },
        Err(err) => panic!("an Error occured! {}", err),
    };
    println!("{:?}", current_dns_ip);

    let current_dns_ip = match dns::lookup::<Ipv4Addr>(Name::from_str(HOSTNAME).unwrap()) {
        Ok(maybe_ip) => match maybe_ip {
            Some(ip) => ip,
            None => panic!("there is no ip")
        },
        Err(err) => panic!("an Error occured! {}", err),
    };
    println!("{:?}", current_dns_ip);

    //if current_dns_ip != new_ip {
    //    match inwx::update_dns(INWX_USER, INWX_PASS, DOMAIN_ID, new_ip) {
    //        Ok(_) => println!("Changed AAAA record to {}", new_ip),
    //        Err(err) => panic!("Error! {}", err),
    //    };
    //}
}
