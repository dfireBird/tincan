use std::io::Read;
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::Receiver;

enum Message {
    Chat,
    File,
}

fn recv_messages(peer: &mut TcpStream, rx: Receiver<(Message, Vec<u8>)>) {
    loop {
        let mut message_type = [0; 4];
        let mut length = [0; 4];
        peer.read(&mut message_type).unwrap();
        peer.read(&mut length).unwrap();
        let length = u32::from_be_bytes(length);
        let mut message = Vec::with_capacity(length as usize);
        peer.read(&mut message).unwrap();

        let message_type = str::from_utf8(&message_type).unwrap();
        match message_type {
            "file" => unimplemented!(), // handle file
            "chat" => unimplemented!(), // handle Chat
            _ => continue,
        }
    }
}
