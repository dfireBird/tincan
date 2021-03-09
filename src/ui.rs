use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
    DEFAULT_PORT,
};

mod commands;

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
                    state.info_message = String::from("A peer is connected");
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

        draw_ui(&mut terminal, &state, &id)?;

        // Handle Input
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Enter => {
                    if state.input.contains("?connect") {
                        commands::connect_command(&mut state, id)?;
                    } else if state.input.contains("?file") {
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
                Span::raw("Use "),
                Span::styled("?connect", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" ip to connect. Press "),
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
