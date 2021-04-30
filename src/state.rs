use std::net::TcpStream;

#[derive(Debug)]
pub enum Author {
    Me,
    Other,
}

pub struct State {
    pub messages: Vec<(Author, String)>,
    pub input: String,
    pub connection: Option<TcpStream>,
    pub connected: bool,
    pub info_message: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            input: String::new(),
            messages: Vec::new(),
            connection: None,
            connected: false,
            info_message: String::new(),
        }
    }
}
