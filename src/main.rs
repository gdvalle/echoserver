extern crate clap;
extern crate hyper;
extern crate tokio;

use clap::{App, Arg};
use futures::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::io::{self, Write};
use std::net::SocketAddr;

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let (parts, body) = req.into_parts();
    let entire_body = body
        .try_fold(Vec::new(), |mut data, chunk| async move {
            data.extend_from_slice(&chunk);
            Ok(data)
        })
        .await
        .unwrap_or("Failed to read body!".as_bytes().to_vec());

    let request_path = match parts.uri.path_and_query() {
        Some(p) => p.as_str(),
        None => parts.uri.path(),
    };

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let output_text = [
        format!("{} {}\n{:?}\n", parts.method, request_path, parts.headers).as_bytes(),
        &entire_body,
        b"\n",
    ]
    .concat();

    if let Err(e) = handle.write_all(&output_text) {
        eprintln!("Failed writing to stdout: {}", e);
    }

    Ok(Response::new(Body::from(entire_body)))
}

// Parse listen arg into array of strings.
fn parse_addresses(listeners: clap::Values) -> Vec<String> {
    let mut addresses = Vec::new();

    for listener in listeners {
        let port = listener.parse::<i32>();
        let addr = match port {
            Ok(v) => match v {
                // It's an int.  Check that it's a valid IPv4 port.
                1..=65535 => format!("0.0.0.0:{}", listener),
                _ => panic!("Bad port for listener: {}", listener),
            },
            Err(_) => format!("{}", listener),
        };
        addresses.push(addr);
    }
    return addresses;
}

#[tokio::main]
async fn main() {
    let args = App::new("echoserver")
        .version("0.0.3")
        .about("HTTP server that prints requests and returns an empty 200.")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .value_name("[IP:]PORT")
                .multiple(false)
                .required(true)
                .help("Bind on this Port or IP:Port")
                .takes_value(true),
        )
        .get_matches();

    let addresses = parse_addresses(args.values_of("listen").unwrap());
    let address = &addresses[0];

    let socket_addr: SocketAddr = address.parse().expect("Failed to parse listen address");
    let server = Server::bind(&socket_addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(echo))
    }));

    println!("Listening on {}", address);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
