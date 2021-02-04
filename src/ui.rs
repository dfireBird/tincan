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
    io::{self, Read, Write},
    net::TcpStream,
};

use super::events::{Event, Events};
use super::state::State;
use super::DEFAULT_PORT;

pub fn start_ui() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut events = Events::new();

    let mut state = State::default();

    // UI loop
    loop {
        draw_ui(&mut terminal, &state)?;

        // Handle Input
        if let Event::Input(input) = events.next()? {
            match input.code {
                KeyCode::Enter => {
                    state.messages.push(state.input.drain(..).collect());
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
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
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
    })?;
    Ok(())
}

fn initiate_connection(state: &mut State, data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let (id, ip) = data.split_at(4);
    let ip = std::str::from_utf8(ip)?;
    let mut message = "Hello".as_bytes().to_vec();
    message.append(&mut id.to_vec());

    let mut connection = TcpStream::connect((ip, DEFAULT_PORT))?;
    connection.write(&message)?;
    let mut recv_message = [0; 9];
    connection.read(&mut recv_message)?;
    if recv_message.to_vec() == message {
        state.connection = Some(connection);
        state.connected = true;
    }
    Ok(())
}
