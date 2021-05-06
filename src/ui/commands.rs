use std::{
    error::Error,
    fs,
    io::{ErrorKind, Write},
    path::Path,
};

use tui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use super::{Author, State};
use crate::widgets::text_message::TextMessage;

pub fn connect_command(state: &mut State, id: u32) -> Result<(), Box<dyn Error>> {
    let input = state.input_box.get_and_clear();
    let (_, ip) = input.split_at(9);
    let mut data = id.to_be_bytes().to_vec();
    data.append(&mut ip.as_bytes().to_vec());
    super::initiate_connection(state, &data)?;
    if state.connected {
        state.info_message = TextMessage::from(Span::styled(
            format!("You're connected to ip: {}", ip),
            Style::default().add_modifier(Modifier::BOLD),
        ));
    } else {
        state.info_message = TextMessage::from(Span::styled(
            "Connection command failed",
            Style::default().fg(Color::Red),
        ));
    }
    Ok(())
}

pub fn send_message(state: &mut State) -> Result<(), Box<dyn Error>> {
    if let Some(connection) = &state.connection {
        let message = state.input_box.get_and_clear();
        let length_bytes = message.len() as u32;
        let mut data = "chat".as_bytes().to_vec();
        data.append(&mut length_bytes.to_be_bytes().to_vec());
        data.append(&mut message.as_bytes().to_vec());
        let mut connection = connection.clone();

        if let Err(error) = connection.write(&data) {
            let result = handle_connection_error(state, error.kind());
            if !result {
                return Ok(());
            }
        }
        state.message_box.add_message(Author::Me, message);
    }
    Ok(())
}

pub fn send_file(state: &mut State) -> Result<(), Box<dyn Error>> {
    let input = state.input_box.get_and_clear();
    let (_, path) = input.split_at(6);
    let path = Path::new(path);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let mut file = match fs::read(path) {
        Ok(file) => file,
        Err(_) => {
            state.info_message = TextMessage::from(Span::styled(
                "File provided can't be accessed",
                Style::default().fg(Color::Red),
            ));
            return Ok(());
        }
    };
    if file.len() > 4096 {
        state.info_message = TextMessage::from(Span::styled(
            "The length of file should be less than 4KB",
            Style::default().fg(Color::Red),
        ));
        return Ok(());
    }
    if file_name.as_bytes().len() > 96 {
        state.info_message = TextMessage::from(Span::styled(
            "The length of file name should be less than 96 bytes",
            Style::default().fg(Color::Red),
        ));
        return Ok(());
    }

    let mut file_name = if file_name.as_bytes().len() == 96 {
        file_name.as_bytes().to_vec()
    } else {
        let capacity = 96 - file_name.as_bytes().len();
        let mut zeroes = vec![0u8; capacity];
        zeroes.append(&mut file_name.as_bytes().to_vec());
        zeroes
    };

    let length = (96 + file.len()) as u32;
    let mut data = "file".as_bytes().to_vec();
    data.append(&mut length.to_be_bytes().to_vec());
    data.append(&mut file_name);
    data.append(&mut file);

    data.append(&mut file);

    if let Some(connection) = &state.connection {
        let mut connection = connection.clone();
        if let Err(error) = connection.write(&data) {
            let _result = handle_connection_error(state, error.kind());
        } else {
            state.info_message = TextMessage::from("The file is sent to peer");
        }
        Ok(())
    } else {
        Ok(())
    }
}

fn handle_connection_error(state: &mut State, kind: ErrorKind) -> bool {
    match kind {
        ErrorKind::ConnectionAborted
        | ErrorKind::ConnectionRefused
        | ErrorKind::ConnectionReset
        | ErrorKind::NotConnected
        | ErrorKind::TimedOut => {
            state.info_message = TextMessage::from(Span::styled(
                "Connection has aborted",
                Style::default().fg(Color::Red),
            ));
            state.connected = false;
            state.connection = None;
            true
        }
        _ => false,
    }
}
