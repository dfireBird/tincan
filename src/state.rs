use std::net::TcpStream;

pub struct State {
    pub messages: Vec<String>,
    pub input: String,
    pub connection: Option<TcpStream>,
    pub connected: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            input: String::new(),
            messages: Vec::new(),
            connection: None,
            connected: false,
        }
    }
}
