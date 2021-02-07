use std::net::TcpStream;

pub struct State {
    pub messages: Vec<String>,
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
