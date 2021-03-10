use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::{Author, State};
pub fn help_message_widget() -> Paragraph<'static> {
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
    Paragraph::new(help_text)
}

pub fn message_box_widget(state: &State, id: &u32) -> List<'static> {
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
    List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"))
}

pub fn input_box_widget(state: &State) -> Paragraph {
    Paragraph::new(state.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"))
}

pub fn info_message_widget(state: &State) -> Paragraph {
    let info = Text::from(Span::styled(
        &state.info_message,
        Style::default().fg(Color::Red),
    ));

    Paragraph::new(info)
}
