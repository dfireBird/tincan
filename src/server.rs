use dirs;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::Sender;

pub enum Message {
    Connect,
    Chat,
    File,
}

pub fn handshake(peer: &mut TcpStream, tx: &Sender<(Message, Vec<u8>)>) -> Option<u32> {
    let mut handshake = [0; 5];
    let mut id = [0; 4];
    peer.read(&mut handshake).unwrap();
    peer.read(&mut id).unwrap();
    if str::from_utf8(&handshake).unwrap() == "Hello" {
        let remote_ip = peer.peer_addr().unwrap().ip().to_string();
        let mut peer_handshake = "Hello".as_bytes().to_vec();
        peer_handshake.append(&mut id.to_vec());

        tx.send((Message::Connect, remote_ip.as_bytes().to_vec()))
            .unwrap();
        peer.write(&peer_handshake).unwrap();
        let id = u32::from_be_bytes(id);
        Some(id)
    } else {
        None
    }
}

pub fn recv_messages(peer: &mut TcpStream, tx: &Sender<(Message, Vec<u8>)>) {
    loop {
        let mut message_type = [0; 4];
        let mut length = [0; 4];
        peer.read(&mut message_type).unwrap();
        peer.read(&mut length).unwrap();
        let length = u32::from_be_bytes(length);
        let mut message = vec![0u8; length as usize];
        peer.read_exact(&mut message).unwrap();

        let message_type = str::from_utf8(&message_type).unwrap();
        match message_type {
            "file" => handle_file(peer, &tx, &message),
            "chat" => handle_chat(&message, tx),
            _ => continue,
        }
    }
}

fn handle_chat(message: &Vec<u8>, tx: &Sender<(Message, Vec<u8>)>) {
    // Send the chat message to UI thread
    tx.send((Message::Chat, message.clone())).unwrap();
}

fn handle_file(peer: &mut TcpStream, tx: &Sender<(Message, Vec<u8>)>, file_data: &Vec<u8>) {
    // Split at 96th byte to get the file name
    let (file_name, file_data) = file_data.split_at(96);

    // If file name if utf8 then write the file or send the error to the other peer
    if let Ok(file_name) = str::from_utf8(file_name) {
        let file_name = file_name.trim_start_matches('\0');
        let mut file_path = dirs::download_dir().unwrap();
        file_path.push(file_name);
        fs::write(&file_path.as_path(), file_data).unwrap();
        tx.send((Message::File, file_name.as_bytes().to_vec()))
            .unwrap();
    } else {
        let error = "File name is not UTF-8 compatabile";
        let error_msg = [
            "Error".as_bytes(),
            &error.as_bytes().len().to_be_bytes(),
            error.as_bytes(),
        ]
        .concat();
        peer.write(&error_msg).unwrap();
    }
}
