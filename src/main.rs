use std::error::Error;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use tincan::server;
use tincan::ui;
use tincan::DEFAULT_PORT;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, DEFAULT_PORT))?;
    let (tx, rx) = mpsc::channel();
    let server_handle = thread::spawn(move || {
        let mut peer: TcpStream;
        loop {
            peer = match listener.accept() {
                Ok((peer, _)) => peer,
                Err(_) => continue,
            };
            match server::handshake(&mut peer, &tx) {
                Some(_) => break,
                None => continue,
            }
        }
        server::recv_messages(&mut peer, &tx);
    });
    ui::start_ui()
}
