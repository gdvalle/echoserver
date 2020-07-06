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

    let headers: Vec<u8> = parts
        .headers
        .iter()
        .map(|(key, value)| {
            [
                key.as_str().as_bytes(),
                &b": "[..],
                value.as_bytes(),
                &b"\n"[..],
            ]
            .concat()
        })
        .flatten()
        .collect();

    let output = [
        format!("{} {}\n", parts.method, request_path).as_bytes(),
        &headers,
        &entire_body,
        b"\n",
    ]
    .concat();

    if let Err(e) = io::stdout().write_all(&output) {
        eprintln!("Failed writing to stdout: {}", e);
    }

    Ok(Response::new(Body::from("")))
}

#[tokio::main]
async fn main() {
    let args = App::new("echoserver")
        .version("0.0.4")
        .about("HTTP server that prints requests and returns an empty 200.")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .value_name("[IP:]PORT")
                .multiple(false)
                .required(true)
                .index(1)
                .help("Bind on this Port or IP:Port")
                .takes_value(true),
        )
        .get_matches();

    let listen_arg = args.value_of("listen").unwrap();
    let address = match listen_arg.contains(":") {
        true => listen_arg.to_string(),
        false => format!("0.0.0.0:{}", listen_arg),
    };

    let socket_addr: SocketAddr = address.parse().expect("Failed to parse listen address");
    let server = Server::bind(&socket_addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(echo))
    }));

    eprintln!("Listening on {}", address);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
