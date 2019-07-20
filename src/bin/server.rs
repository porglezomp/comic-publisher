use futures::{future, Future};
use hyper;
use hyper_staticfile::Static;
use std::io::Error;

fn main() {
    let addr = ([127, 0, 0, 1], 8888).into();
    let server = hyper::Server::bind(&addr)
        .serve(|| future::ok::<_, Error>(Static::new("output/")))
        .map_err(|e| eprintln!("server error: {}", e));
    eprintln!("Hosting your website at http://{}/.", addr);
    hyper::rt::run(server);
}
