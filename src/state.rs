use std::net::TcpStream;

use tui::text::Span;

use super::widgets::{input_box::InputBox, message_box::MessageBox, text_message::TextMessage};

#[derive(Debug, Clone)]
pub enum Author {
    Me,
    Other,
}

pub struct State<'a> {
    pub message_box: MessageBox,
    pub input_box: InputBox,
    pub connection: Option<TcpStream>,
    pub connected: bool,
    pub info_message: TextMessage<'a>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self {
            input_box: InputBox::new(),
            message_box: MessageBox::new(),
            connection: None,
            connected: false,
            info_message: TextMessage::from(Span::raw("")),
        }
    }
}
