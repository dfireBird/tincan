use tui::{
    buffer::Buffer,
    layout::Rect,
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

struct TextMessage<'a> {
    text: Spans<'a>,
}

impl<'a> Widget for TextMessage<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.text).render(area, buf)
    }
}

impl<'a> From<Vec<Span<'a>>> for TextMessage<'a> {
    fn from(spans: Vec<Span<'a>>) -> Self {
        Self {
            text: Spans::from(spans),
        }
    }
}

impl<'a> From<Span<'a>> for TextMessage<'a> {
    fn from(span: Span<'a>) -> Self {
        Self {
            text: Spans::from(span),
        }
    }
}

impl<'a> From<&'a str> for TextMessage<'a> {
    fn from(str: &'a str) -> Self {
        Self {
            text: Spans::from(str),
        }
    }
}

impl<'a> From<String> for TextMessage<'a> {
    fn from(str: String) -> Self {
        Self {
            text: Spans::from(str),
        }
    }
}
