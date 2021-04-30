use tui::{
    buffer::Buffer,
    layout::Rect,
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::state::Author;

#[derive(Debug)]
struct MessageBox {
    messages: Vec<(Author, String)>,
    _id: u32, // TODO: change _id to have proper reference
}

impl Widget for MessageBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        List::new(self.messages_listitem())
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .render(area, buf);
    }
}

impl MessageBox {
    fn messages_listitem(&self) -> Vec<ListItem> {
        self.messages
            .iter()
            .map(|(a, m)| {
                ListItem::new(match a {
                    Author::Me => vec![Spans::from(Span::raw(format!("me: {}", m)))],
                    Author::Other => vec![Spans::from(Span::raw(format!("{}: {}", self._id, m)))],
                })
            })
            .collect()
    }

    pub fn add_message(&mut self, author: Author, message: String) {
        self.messages.push((author, message));
    }
}
