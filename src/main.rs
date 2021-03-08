use std::{
    error::Error,
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

use tincan::{server, ui, DEFAULT_PORT};

fn main() -> Result<(), Box<dyn Error>> {
    let id = tincan::generate_id();
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, DEFAULT_PORT))?;
    let (tx, rx) = mpsc::channel();
    let _server_handle = thread::spawn(move || {
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
    ui::start_ui(id, &rx)
}
