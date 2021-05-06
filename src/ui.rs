use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    Terminal,
};

use std::{
    error::Error,
    io::{self, Read, Stdout, Write},
    net::TcpStream,
    sync::mpsc::Receiver,
};

use super::{
    events::{Event, Events},
    server::Message,
    state::{Author, State},
    widgets::text_message::TextMessage,
    DEFAULT_PORT,
};

mod commands;
mod widgets;

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
                    state.info_message = TextMessage::from(Span::styled(
                        "A peer is connected",
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                    Ok(())
                }
                Message::Chat => recv_chat(&mut state, &data),
                Message::File => {
                    let file_name = String::from_utf8_lossy(&data).to_string();
                    state.info_message = TextMessage::from(Span::styled(
                        format!("File was recieved with file name: {}", file_name),
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                    Ok(())
                }
                _ => Ok(()),
            }?;
        }

        draw_ui(&mut terminal, &state, &id)?;

        // Handle Input
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Enter => {
                    if state.input_box.get().contains("?connect") {
                        commands::connect_command(&mut state, id)?;
                    } else if state.input_box.get().contains("?file") {
                        commands::send_file(&mut state)?;
                    } else {
                        commands::send_message(&mut state)?;
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
                    state.input_box.add_char(c);
                }
                KeyCode::Backspace => {
                    state.input_box.remove_char();
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
        f.render_widget(
            TextMessage::from(vec![
                Span::raw("Use "),
                Span::styled("?connect", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" ip to connect. Press "),
                Span::styled("Ctrl-D", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit."),
            ]),
            chunks[0],
        );

        // Messages Block
        // let messages = widgets::message_box_widget(&state.messages, id);
        f.render_widget(state.message_box.clone(), chunks[1]);

        // Input Block
        // let input = widgets::input_box_widget(&state.input);
        f.render_widget(state.input_box.clone(), chunks[2]);

        // Info message block
        // let info = widgets::info_message_widget(&state.info_message);
        f.render_widget(state.info_message.clone(), chunks[3]);
    })?;
    Ok(())
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
            state.info_message = TextMessage::from(Span::styled(
                "Peer has not started the application or there is no route to peer",
                Style::default().fg(Color::Red),
            ));
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
    state.message_box.add_message(Author::Other, message);
    Ok(())
}
