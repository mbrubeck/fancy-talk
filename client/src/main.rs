extern crate fancy_talk;
extern crate ansi_term;
use std::env;
use std::net::UdpSocket;
use std::process;
use std::str;

use fancy_talk::{Package, Encoder, Decoder, Serialisable};
use ansi_term::Color::RGB;

const MAX_UDP_SIZE : usize = 4096;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(2);
    });

    let socket = UdpSocket::bind("127.0.0.1:65433").unwrap();

    let mut query = Package::new();
    query.query = Some(config.query);


    let mut out_buf: Vec<u8> = Vec::new();

    {
        let mut encoder = Encoder::new(&mut out_buf);
        query.write(&mut encoder).expect("Failed to encode query");
    }

    let sent_size = socket.send_to(out_buf.as_slice(), (config.address.as_str(), config.port)).expect("Failed to send data to server");
    println!("Sent {} bytes", sent_size);
    let mut in_buf : [u8; MAX_UDP_SIZE] = [0; MAX_UDP_SIZE];
    let amt = socket.recv(&mut in_buf).expect("Reading from server failed");
    println!("Got {} bytes.", amt);

    let mut decoder = Decoder::new(&in_buf);

    let response = Package::read(&mut decoder).expect("Parsing the response failed");

    let mut outstyle = RGB(response.red, response.green, response.blue).normal();
    if response.bold {
        outstyle = outstyle.bold();
    }
    if response.italic {
        outstyle = outstyle.italic();
    }
    if response.underlined {
        outstyle = outstyle.underline();
    }
    if response.blink {
        outstyle = outstyle.blink();
    }

    let res_text = match response.payload {
        None => String::from("<empty>"),
        Some(text) => text,
    };

    println!("{}", outstyle.paint(res_text));

}

struct Config {
    address: String,
    port: u16,
    query: String,
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {

        if args.len() < 4 {
            return Err("not enough arguments")
        }

        let address = args[1].clone();
        let port = args[2].parse::<u16>().unwrap();
        let query = args[3].clone();

        Ok(Config { address, port, query })
    }

}
