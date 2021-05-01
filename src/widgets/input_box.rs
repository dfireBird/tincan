use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug)]
struct InputBox {
    input: String,
}

impl Widget for InputBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.input)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Input"))
            .render(area, buf)
    }
}

impl InputBox {
    pub fn add_char(&mut self, c: char) {
        self.input.push(c)
    }

    pub fn remove_char(&mut self) {
        self.input.pop();
    }

    pub fn get_and_clear(&mut self) -> String {
        self.input.drain(..).collect()
    }

    pub fn get(&mut self) -> String {
        self.input.clone()
    }
}
