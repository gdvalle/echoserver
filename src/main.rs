extern crate iron;
extern crate clap;
extern crate chan_signal;

use clap::{Arg, App};
use iron::prelude::{Iron, IronResult, Request, Response};
use std::io::Read;
use iron::status;
use chan_signal::Signal;

fn echo_handler(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    let body_size = match req.body.read_to_string(&mut body) {
        Ok(x) => x,
        Err(_) => 0,
    };

    match body_size {
        0 => println!("{} {}", req.method, req.url),
        _ => println!("{} {}\n{}", req.method, req.url, body),
    };

    Ok(Response::with(status::Ok))
}

// Parse listen arg into array of strings.
fn parse_addresses(listeners: clap::Values) -> Vec<String> {
    let mut addresses = Vec::new();

    for listener in listeners {
        let port = listener.parse::<i32>();
        let addr = match port {
            Ok(v) => match v {
                // It's an int.  Check that it's a valid IPv4 port.
                1...65535 => format!("0.0.0.0:{}", listener),
                _ => panic!("Bad port for listener: {}", listener),
            },
            Err(_) => format!("{}", listener),
        };
        addresses.push(addr);
    };
    return addresses;
}

fn main() {
    let args = App::new("echoserver")
        .version("0.0.2")
        .about("HTTP server that prints requests and returns an empty 200.")
        .arg(Arg::with_name("listen")
            .short("l")
            .long("listen")
            .value_name("[IP:]PORT")
            .multiple(true)
            .required(true)
            .help("Bind on this Port or IP:Port")
            .takes_value(true))
        .get_matches();

    let addresses = parse_addresses(args.values_of("listen").unwrap());

    let mut servers = Vec::new();

    for addr in addresses {
        println!("Listening on {}", addr);
        // To not block on this call it must be stored somewhere?
        let server = Iron::new(echo_handler).http(&*addr).unwrap();
        servers.push(server);
    };

    println!("Started {} servers", servers.len());

    // Wait for SIGINT/SIGTERM.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    signal.recv().unwrap();
}