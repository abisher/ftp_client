extern crate ftp_server;

use std::fs::read_dir;
use ftp_server::enums::{ResultCode, Command};
use ftp_server::utils::add_file_info;

use std::net::{TcpListener, TcpStream, IpAddr, Ipv6Addr, SocketAddr, Ipv4Addr};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{env, io, thread};

#[allow(dead_code)]
struct Client {
    cwd: PathBuf,
    stream: TcpStream,
    name: Option<String>,
    data_writer: Option<TcpStream>,
}

impl Client {
    fn new(stream: TcpStream) -> Self {
        Self {
            cwd: PathBuf::from("/"),
            stream,
            name: None,
            data_writer: None,
        }
    }

    fn handle_cmd(&mut self, cmd: Command) {
        println!("====> {:?}", cmd);

        match cmd {
            Command::Auth => send_cmd(&mut self.stream, ResultCode::CommandNotImplemented, "Not implemented"),
            Command::Syst => send_cmd(&mut self.stream, ResultCode::Ok, "I won't tell"),
            Command::Unknown(s) => send_cmd(&mut self.stream, ResultCode::UnknownCommand, "Unknown"),
            Command::User(username) => {
                if username.is_empty() {
                    send_cmd(&mut self.stream, ResultCode::InvalidParameterOrArgument, "Invalid username")
                } else {
                    self.name = Some(username.to_owned());
                    send_cmd(&mut self.stream, ResultCode::UserLoggedIn,
                             &format!("Welcome {}!", username))
                }
            }
            Command::Noop => send_cmd(&mut self.stream, ResultCode::Ok, "Doing
nothing..."),
            Command::Pwd => {
                let msg = format!("{}", self.cwd.to_str().unwrap_or(""));
                if !msg.is_empty() {
                    let message = format!("\"/{}\"", msg);
                    send_cmd(&mut self.stream, ResultCode::PATHNAMECreated, &message);
                } else {
                    send_cmd(&mut self.stream, ResultCode::FileNotFound, "No such file or directory")
                }
            }
            Command::Type => send_cmd(&mut self.stream, ResultCode::Ok, "Transfer type changed \
            successfully"),
            Command::Pasv => {
                if self.data_writer.is_some() {
                    send_cmd(&mut self.stream, ResultCode::DataConnectionAlreadyOpen,
                             "Already listening")
                } else {
                    let port: u16 = 43210;
                    send_cmd(&mut self.stream, ResultCode::EnteringPassiveMode,
                             &format!("127,0,0,1,{},{}", port >> 8, port & 0xFF));
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
                    let listener = TcpListener::bind(&addr).unwrap();

                    match listener.incoming().next() {
                        Some(Ok(client)) => {
                            self.data_writer = Some(client);
                        }
                        _ => send_cmd(&mut self.stream, ResultCode::ServiceNotAvailable, "issues happen...")
                    }
                }
            }

            Command::List(path) => {
                if let Some(ref mut data_writer) = self.data_writer {
                    let mut tmp = PathBuf::from(".");
                    send_cmd(&mut self.stream, ResultCode::DataConnectionAlreadyOpen,
                             "Starting to list directory");

                    let mut out = String::new();
                    for entry in read_dir(tmp).unwrap() {
                        // for entry in dir {
                        if let Ok(entry) = entry {
                            add_file_info(entry.path(), &mut out)
                        }
                        //}
                        send_data(data_writer, &out);
                    }
                } else {
                    send_cmd(&mut self.stream, ResultCode::ConnectionClosed,
                             "No opened data connection");
                }
                if self.data_writer.is_some() {
                    self.data_writer = None;
                    send_cmd(&mut self.stream, ResultCode::ClosingDataConnection, "Transfer done!");
                }
            }
            Command::CdUp => {
                if let Some(path) = self.cwd.parent().map(Path::to_path_buf) {
                    self.cwd = path;
                }
                send_cmd(&mut self.stream, ResultCode::Ok, "Done");
            }
            _ => (),
        }
    }

    fn complete_path(&self, path: PathBuf, server_root: &PathBuf) ->
    Result<PathBuf, io::Error> {
        let directory = server_root.join(if path.has_root() {
            path.iter().skip(1).collect()
        } else {
            path
        });

        let dir = directory.canonicalize();

        if let Ok(ref dir) = dir {
            if !dir.starts_with(&server_root) {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        dir
    }

    fn cwd(mut self, directory: PathBuf) {
        let server_root = env::current_dir().unwrap();
        let path = self.cwd.join(&directory);

        if let Ok(dir) = self.complete_path(path, &server_root) {
            if let Ok(prefix) = dir.strip_prefix(&server_root)
                .map(|p| p.to_path_buf()) {
                self.cwd = prefix.to_path_buf();
                send_cmd(&mut self.stream, ResultCode::Ok,
                         &format!("Directory changed to \"{}\"", directory.display()))
            }
            return;
        }
        send_cmd(&mut self.stream, ResultCode::FileNotFound, "No such file or directory");
    }
}

fn handle_client(mut stream: TcpStream) {
    println!("new client connected!");
    send_cmd(&mut stream, ResultCode::ServiceReadyForNewUser, "Welcome to this FTP server!");

    let mut client = Client::new(stream);

    loop {
        let data = read_all_message(&mut client.stream);

        if data.is_empty() {
            println!("Client disconnected!");
            break;
        }
        if let Ok(command) = Command::new(data) {
            client.handle_cmd(command)
        } else {
            println!("Error with client command!");
        }
    }
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

fn read_all_message(stream: &mut TcpStream) -> Vec<u8> {
    let buf = &mut [0; 1];
    let mut out = Vec::with_capacity(100);

    loop {
        match stream.read(buf) {
            Ok(received)if received > 0 => {
                if out.is_empty() && buf[0] == b' ' {
                    continue;
                }
                out.push(buf[0])
            }
            _ => return Vec::new(),
        }
        let len = out.len();
        if len > 1 && out[len - 2] == b'\r' && out[len - 1] == b'\n' {
            out.pop();
            out.pop();
            return out;
        }
    }
}

fn send_data(stream: &mut TcpStream, s: &str) {
    write!(stream, "{}", s).unwrap();
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


#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::read_dir;

    #[test]
    fn test_some() {
        for entry in read_dir("target").unwrap() {
            let some = entry.unwrap();
            let path = some.path();
            println!("{:?} - {:?}", path, path.is_dir());
        }
        assert!(true)
    }

    #[test]
    fn dich() {
        println!("{:?}", env::current_dir());
        assert!(true)
    }
}