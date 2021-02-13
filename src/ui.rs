use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::LeaveAlternateScreen,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Modifier,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::Block,
    widgets::Borders,
    widgets::List,
    widgets::ListItem,
    widgets::Paragraph,
    Terminal,
};

use std::{
    error::Error,
    fs,
    io::{self, ErrorKind, Read, Write},
    net::TcpStream,
    path::Path,
    sync::mpsc::Receiver,
};

use super::events::{Event, Events};
use super::server::Message;
use super::state::Author;
use super::state::State;
use super::DEFAULT_PORT;

pub fn start_ui(id: u32, rx: &Receiver<(Message, Vec<u8>)>) -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut state = State::default();

    // UI loop
    loop {
        for (message, data) in rx.try_iter() {
            match message {
                Message::Connect if !state.connected => {
                    let mut data_ = id.to_be_bytes().to_vec();
                    data_.append(&mut data.to_owned());
                    let result = initiate_connection(&mut state, &data_);
                    if let Err(_) = result {
                        terminal_deinitialization(&mut terminal)?;
                    }
                    Ok(())
                }
                Message::Chat => recv_chat(&mut state, &data),
                Message::File => {
                    let file_name = String::from_utf8_lossy(&data).to_string();
                    state.info_message = format!("File was recieved with file name: {}", file_name);
                    Ok(())
                }
                _ => Ok(()),
            }?;
        }

        if state.connected {
            state.info_message = String::from("A peer is connected");
        }

        draw_ui(&mut terminal, &state, &id)?;

        // Handle Input
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Enter => {
                    if state.input.contains("?connect") {
                        connect_command(&mut state, id)?;
                    } else if state.input.contains("?file") {
                        send_file(&mut state)?;
                    } else {
                        send_message(&mut state)?;
                    }
                }
                KeyCode::Char(c) if c == 'd' && input.modifiers == KeyModifiers::CONTROL => {
                    terminal_deinitialization(&mut terminal)?;
                    break;
                }
                KeyCode::Char(c) if c == 'c' && input.modifiers == KeyModifiers::CONTROL => {
                    terminal_deinitialization(&mut terminal)?;
                    break;
                }
                KeyCode::Char(c) => {
                    state.input.push(c);
                }
                KeyCode::Backspace => {
                    state.input.pop();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn terminal_deinitialization(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_ui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &State,
    id: &u32,
) -> Result<(), Box<dyn Error>> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(f.size());

        // Help Message
        let (msg, style) = (
            vec![
                Span::raw("Press "),
                Span::styled("Ctrl-D", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        );
        let mut help_text = Text::from(Spans::from(msg));
        help_text.patch_style(style);
        f.render_widget(Paragraph::new(help_text), chunks[0]);

        // Messages Block
        let messages: Vec<ListItem> = state
            .messages
            .iter()
            .map(|(a, m)| {
                let content = match a {
                    Author::Me => vec![Spans::from(Span::raw(format!("me: {}", m)))],
                    Author::Other => vec![Spans::from(Span::raw(format!("{}: {}", id, m)))],
                };

                ListItem::new(content)
            })
            .collect();
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        f.render_widget(messages, chunks[1]);

        // Input Block
        let input = Paragraph::new(state.input.as_ref())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[2]);

        // Info message block
        let info = Text::from(Span::styled(
            &state.info_message,
            Style::default().fg(Color::Red),
        ));
        f.render_widget(Paragraph::new(info), chunks[3]);
    })?;
    Ok(())
}

fn connect_command(state: &mut State, id: u32) -> Result<(), Box<dyn Error>> {
    let input: String = state.input.drain(..).collect();
    let (_, ip) = input.split_at(9);
    let mut data = id.to_be_bytes().to_vec();
    data.append(&mut ip.as_bytes().to_vec());
    initiate_connection(state, &data)?;
    if state.connected {
        state.info_message = format!("You're connected to ip: {}", ip);
    } else {
        state.info_message = String::from("Connection command failed");
    }
    Ok(())
}

fn send_message(state: &mut State) -> Result<(), Box<dyn Error>> {
    if let Some(connection) = &state.connection {
        let message: String = state.input.drain(..).collect();
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
        state.messages.push((Author::Me, message));
    }
    Ok(())
}

fn send_file(state: &mut State) -> Result<(), Box<dyn Error>> {
    let input: String = state.input.drain(..).collect();
    let (_, path) = input.split_at(6);
    let path = Path::new(path);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let mut file = match fs::read(path) {
        Ok(file) => file,
        Err(_) => {
            state.info_message = String::from("File provided can't be accessed");
            return Ok(());
        }
    };
    if file.len() > 4096 {
        state.info_message = String::from("The length of file should be less than 4KB");
        return Ok(());
    }
    if file_name.as_bytes().len() > 96 {
        state.info_message = String::from("The length of file name should be less than 96 bytes");
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
            state.info_message = String::from("The file is sent to the peer");
        }
        Ok(())
    } else {
        Ok(())
    }
}

fn initiate_connection(state: &mut State, data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let (id, ip) = data.split_at(4);
    let ip = std::str::from_utf8(ip)?;
    let mut message = "Hello".as_bytes().to_vec();
    message.append(&mut id.to_vec());

    let mut connection: TcpStream;
    match TcpStream::connect((ip, DEFAULT_PORT)) {
        Ok(conn) => connection = conn,
        Err(_) => {
            state.info_message =
                String::from("Peer has not started the application or there is no route to peer");
            return Ok(());
        }
    };
    connection.write(&message)?;
    let mut recv_message = [0; 9];
    connection.read(&mut recv_message)?;
    if recv_message.to_vec() == message {
        state.connection = Some(connection);
        state.connected = true;
    }
    Ok(())
}

fn recv_chat(state: &mut State, data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let message = String::from_utf8(data.to_owned())?;
    state.messages.push((Author::Other, message));
    Ok(())
}

fn handle_connection_error(state: &mut State, kind: ErrorKind) -> bool {
    match kind {
        ErrorKind::ConnectionAborted
        | ErrorKind::ConnectionRefused
        | ErrorKind::ConnectionReset
        | ErrorKind::NotConnected
        | ErrorKind::TimedOut => {
            state.info_message = String::from("Connection has aborted");
            state.connected = false;
            state.connection = None;
            true
        }
        _ => false,
    }
}
