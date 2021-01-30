use dirs;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::mpsc::Sender;

enum Message {
    Chat,
    File,
}

fn recv_messages(peer: &mut TcpStream, tx: &Sender<(Message, Vec<u8>)>) {
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
            "file" => handle_file(peer, &message),
            "chat" => handle_chat(&message, tx),
            _ => continue,
        }
    }
}

fn handle_chat(message: &Vec<u8>, tx: &Sender<(Message, Vec<u8>)>) {
    // Send the chat message to UI thread
    tx.send((Message::Chat, message.clone())).unwrap();
}

fn handle_file(peer: &mut TcpStream, file_data: &Vec<u8>) {
    // Split at 96th byte to get the file name
    let (file_name, file_data) = file_data.split_at(96);

    // If file name if utf8 then write the file or send the error to the other peer
    if let Ok(file_name) = str::from_utf8(file_name) {
        let mut file_path = dirs::download_dir().unwrap();
        file_path.push(file_name);
        fs::write(&file_path.as_path(), file_data).unwrap();
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
