use crate::character::{self, Character};
use crate::gamemap::{self, GameMap};
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

pub struct RogueGame {
    map: GameMap,
    character: Character,
}

impl RogueGame {
    pub fn new(width: i16, height: i16) -> Self {
        RogueGame {
            map: GameMap::new(width, height),
            character: Character::new(),
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('w') => self.character.move_up(),
            _ => {}
        }
    }

    pub fn to_text(&self) -> Text<'_> {
        self.map.to_text()
    }
}

impl Widget for &RogueGame {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" idle game yass ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Paragraph::new(self.to_text())
            .centered()
            .block(block)
            .render(area, buf);
    }
}
