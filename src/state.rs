pub struct State {
    pub messages: Vec<String>,
    pub input: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            input: String::new(),
            messages: Vec::new(),
        }
    }
}
