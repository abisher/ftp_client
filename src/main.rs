extern crate ftp_server;

use ftp_server::enums::{ResultCode, Command};

use std::net::{TcpListener, TcpStream};
use std::io::Write;
use std::thread;


fn handle_client(mut stream: TcpStream) {
    println!("new client connected!");
    // client code handling here
}


fn send_cmd(stream: &mut TcpStream, code: ResultCode, message: &str) {
    let msg = if message.is_empty() {
        format!("{}\r\n", code as u32)
    } else {
        format!("{} {}\r\n", code as u32, message)
    };
    println!("<==== {}", msg);
    write!(stream, "{}", msg).unwrap()
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1234")
        .expect("Couldn't bind address");

    println!("Waiting for clients to connect!");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
            _ => println!("A client tried to connect")
        }
    }


}


