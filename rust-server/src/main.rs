extern crate fancy_talk;
use std::net::UdpSocket;
use std::collections::HashMap;
use fancy_talk::{Package, MessageType, Decoder, Encoder, Serialisable};


const MAX_UDP_SIZE : usize = 4096;

fn lookup_message(messages: &HashMap<&str, Package>, query: &Package) -> Package {

    let query_text = match query.query {
        None => String::from("fallback"),
        Some(ref text) => String::from(text.as_str()),
    };

    let resp_query = match query.query {
        None => None,
        Some(ref text) => Some(String::from(text.as_str())),
    };
    let mut resp = match messages.get(query_text.as_str()) {
        None => messages.get("fallback").unwrap().clone(),
        Some(response) => response.clone(),
    };
    resp.id = query.id;
    resp.query = resp_query;
    resp
}

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:65432").expect("Binding to the socket failed");
    let greeting = Package::new().set_italic(true).set_rgb(0xEE, 0x66, 0x22)
                                 .set_payload(Some(String::from("Hello, world!")))
                                 .set_message_type(MessageType::Response);
    let hamlet = Package::new().set_underlined(true).set_rgb(0x00, 0x66, 0x66)
                               .set_payload(Some(String::from("Alas, poor Yorick!")))
                               .set_message_type(MessageType::Response);
    let farewell = Package::new().set_bold(true).set_rgb(0x00, 0x22, 0x66)
                                 .set_payload(Some(String::from("Time to sahay goooooodbyeeeeeee!!!!")))
                                 .set_message_type(MessageType::Response);
    let fallback = Package::new().set_bold(true).set_blink(true).set_rgb(0xff, 0x00, 0x00)
                                 .set_message_type(MessageType::Response)
                                 .set_payload(Some(String::from("Not found!")));

    let mut messages = HashMap::new();
    messages.insert("greeting", greeting);
    messages.insert("hamlet", hamlet);
    messages.insert("farewell", farewell);
    messages.insert("fallback", fallback);

    loop {
        let mut buf : [u8; MAX_UDP_SIZE] = [0; MAX_UDP_SIZE];
        let (amt, src) = socket.recv_from(&mut buf).expect("Recv from socket failed");
        let buf = &mut buf[..amt];
        let mut decoder = Decoder::new(buf);

        let query = Package::read(&mut decoder).expect("Parsing query failed");
        let response = lookup_message(&mut messages, &query);

        let mut outbuf: Vec<u8> = Vec::new();

        {
            let mut encoder = Encoder::new(&mut outbuf);
            response.write(&mut encoder).expect("Encoding response failed");
        }

        socket.send_to(outbuf.as_slice(), &src).expect("Sending reply failed");
    }
}
