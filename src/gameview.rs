use crate::map::{self, Map};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Span, Text, ToSpan},
    widgets::{Block, Paragraph, Widget},
};

pub struct Position(i16, i16);

pub struct GameView {
    char_position: Position,
    map: Map,
}

impl GameView {
    pub fn new(width: i16, height: i16) -> Self {
        GameView {
            char_position: Position(0, 0),
            map: Map::new(width, height),
        }
    }

    pub fn to_text(&self) -> Text<'_> {
        self.map.to_text()
    }
}

impl Widget for &GameView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.to_text().render(area, buf);
    }
}
