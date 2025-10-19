use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

pub struct Position(i16, i16);

pub struct GameView<'a> {
    char_position: Position,
    view_contents: Paragraph<'a>,
}

impl GameView<'_> {
    pub fn new() -> Self {
        GameView {
            char_position: Position(0, 0),
            view_contents: Paragraph::new(""),
        }
    }

    pub fn render_view(&mut self) {
        let height = 20;
        let width = 20;
        let mut lines: Vec<Line> = Vec::new();

        let bg_char = Span::from(".").gray();

        for _ in 0..height {
            let mut chars: Vec<Span> = Vec::new();
            for _ in 0..width {
                chars.push(bg_char.clone());
            }
            lines.push(Line::from(chars));
        }

        self.view_contents = Paragraph::new(lines);
    }
}

impl Widget for &GameView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let view_contents = &self.view_contents;
        view_contents.render(area, buf)
    }
}
