use crate::character::{self, Character, Direction};
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
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub struct RogueGame {
    map: Rc<RefCell<GameMap>>,
    character: Rc<RefCell<Character>>,
}

impl RogueGame {
    pub fn new(width: usize, height: usize) -> Self {
        let character = Rc::new(RefCell::new(Character::new()));
        let map = Rc::new(RefCell::new(GameMap::new(width, height)));

        // Set up the circular references
        character.borrow_mut().set_map(Rc::clone(&map));
        map.borrow_mut().set_character(Rc::clone(&character));

        RogueGame { map, character }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') => self.character.borrow_mut().move_direction(Direction::DOWN),
            KeyCode::Char('w') => self.character.borrow_mut().move_direction(Direction::UP),
            KeyCode::Char('d') => self.character.borrow_mut().move_direction(Direction::RIGHT),
            KeyCode::Char('a') => self.character.borrow_mut().move_direction(Direction::LEFT),
            _ => {}
        }
    }

    pub fn to_text(&self) -> Text<'static> {
        self.map.borrow().to_text()
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
