mod inwx;
mod dns;
mod config;

extern crate reqwest;
extern crate trust_dns;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

extern crate hyper;
extern crate futures;

#[macro_use]
extern crate quick_error;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;

use trust_dns::rr::Name;

use futures::Future;
use futures::Stream;
use hyper::server::{Http, Request, Response, Service};
use hyper::header::ContentLength;
use hyper::{Method, StatusCode};

use config::Config;

static USAGE: &'static str = "Proxy for setting DNS entries at INWX

Usage:
    inxwdynserver [CONFIGPATH]";

type TokenMap = HashMap<String, (Name, u32)>;

struct UpdateService {
    tokens: Arc<TokenMap>,
    inwx_username: String,
    inwx_password: String,
}

impl UpdateService {
    fn new(inwx_username: String, inwx_password: String, tokens: Arc<TokenMap>) -> UpdateService {
        UpdateService{tokens: tokens, inwx_username: inwx_username, inwx_password: inwx_password}
    }
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
                                             .with_body("try POSTing data")))
            },
            (&Method::Post, "/update") => {
                let new_addr = match req.headers().get_raw("X-Real-IP") {
                    Some(real_ip_raw) => {
                        let addr_str = &String::from_utf8_lossy(real_ip_raw.one().unwrap());
                        let addr_str = addr_str.split(',').nth(0).unwrap();
                        IpAddr::from_str(addr_str).unwrap()
                    },
                    None => req.remote_addr().unwrap().ip(),  // TODO: when will this fail?
                };

                // we need to clone() here, because of the move closure
                // TODO: can we solve this any better? These are all only needed read-only...
                let tokens = self.tokens.clone();
                let username = self.inwx_username.clone();
                let password = self.inwx_password.clone();

                Box::new(req.body().concat2().map(move |b| {
                    let token = String::from_utf8_lossy(b.as_ref().into());
                    println!("{}", token);
                    let (hostname, domain_id) = match tokens.get(&*token) {
                        Some(&(ref hostname, domain_id)) => (hostname, domain_id),
                        None => {
                            return Response::new()
                                .with_status(StatusCode::Unauthorized)
                                .with_body("NACK unknown token")},
                    };

                    let changed = match new_addr {
                        IpAddr::V4(new_ip) => {
                            let old_ip = dns::lookup::<Ipv4Addr>(hostname);
                            match old_ip {
                                Ok(Some(ip)) => new_ip != ip,
                                // TODO: log errors?
                                _ => true,
                            }
                        },
                        IpAddr::V6(new_ip) => {
                            let old_ip = dns::lookup::<Ipv6Addr>(hostname);
                            match old_ip {
                                Ok(Some(ip)) => new_ip != ip,
                                // TODO: log errors?
                                _ => true,
                            }
                        },
                    };

                    if changed {
                        match inwx::update_dns(&username, &password, domain_id, new_addr) {
                            Ok(_) => println!("Changed ip to {}", new_addr),
                            Err(err) => {
                                println!("Error during DNS update: {}", err);
                                return Response::new()
                                    .with_status(StatusCode::InternalServerError)
                                    .with_body("NACK could not update DNS record")
                            },
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

// create a token -> domain map from the config for easier lookup
fn get_tokens(config: &Config) -> TokenMap {
    let mut tokens = TokenMap::new();
    for host in config.hosts.iter() {
        tokens.insert((*host.token).to_owned(), (host.domain_name.clone(), host.domain_id));
    }
    tokens
}

fn main() {
    let configfile = std::env::args().nth(1).unwrap_or("config.toml".to_owned());
    if configfile == "--help" {
        eprintln!("{}", USAGE);
        std::process::exit(0);
    }

    let conf = match config::get_config(configfile) {
        Ok(conf) => conf,
        Err(err) => {
            eprintln!("Could not read configuration: {}", err);
            std::process::exit(1);
        }
    };
    let tokens = get_tokens(&conf);
    let token_pt = Arc::new(tokens);

    let username = conf.inwx.username;
    let password = conf.inwx.password;

    let addr = SocketAddr::new(conf.api.bind, conf.api.port);
    let server = Http::new().bind(&addr, move || Ok(UpdateService::new(username.clone(), password.clone(), token_pt.clone()))).unwrap();
    server.run().unwrap()
}
