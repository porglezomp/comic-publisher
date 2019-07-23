use futures::{future, Future};
use hyper::{self, service::Service, Body};
use hyper_staticfile::{Static, StaticFuture};
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    io::{self},
};
use toml;

#[derive(Deserialize)]
struct Config {
    #[serde(default = "String::default")]
    base_path: String,
}

struct Server {
    root: String,
    server: Static,
}

impl Service for Server {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = io::Error;
    type Future = StaticFuture<Body>;

    fn call(&mut self, mut req: hyper::Request<Self::ReqBody>) -> Self::Future {
        let mut path = req.uri().path();
        if path.starts_with(&self.root) {
            path = &path[self.root.len()..];
        } else if path.starts_with(&format!("/{}", &self.root)) {
            path = &path[self.root.len() + 1..];
        }
        *req.uri_mut() = hyper::Uri::builder().path_and_query(path).build().unwrap();
        self.server.call(req)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let addr = ([127, 0, 0, 1], 8888).into();
    let config_text = fs::read_to_string("input/config.toml")
        .map_err(|err| format!("input/config.toml not found: {}", err))?;
    let config: Config = toml::from_str(&config_text)
        .map_err(|err| format!("error parsing input/config.toml: {}", err))?;
    let path = config.base_path.clone();
    let server = hyper::Server::bind(&addr)
        .serve(move || {
            future::ok::<_, io::Error>(Server {
                root: config.base_path.clone(),
                server: Static::new("output/"),
            })
        })
        .map_err(|e| eprintln!("Server error: {}", e));
    eprintln!("Hosting your website at http://localhost:8888/{}", path);
    hyper::rt::run(server);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("Error: {}", err);
        println!("Press [enter] to finish.");
        io::stdin().read_line(&mut String::new()).unwrap();
    }
}
