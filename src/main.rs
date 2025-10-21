use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Clear, Widget},
};

use crate::roguegame::RogueGame;

mod character;
mod effects;
mod enemy;
mod roguegame;
mod weapon;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    game_view: Option<RogueGame>,
    exit: bool,
    tick_rate: Duration,
}

impl App {
    pub fn new() -> App {
        App {
            game_view: None,
            exit: false,
            tick_rate: Duration::from_millis(10),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = self.tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                self.handle_events()?;
            }
            if last_tick.elapsed() >= self.tick_rate {
                if let Some(ref mut game_view) = self.game_view {
                    game_view.update();
                    if game_view.game_over {
                        self.game_view = None;
                    }
                }
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());
        if let Some(ref game_view) = self.game_view {
            frame.render_widget(game_view, frame.area());
        } else {
            frame.render_widget(self, frame.area());
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if let Some(ref mut game_view) = self.game_view {
                    game_view.handle_key_event(key_event);
                }
                self.handle_key_event(key_event)
            }

            _ => {}
        };
        Ok(())
    }

    fn start_game(&mut self) {
        self.game_view = Some(RogueGame::new(100, 50))
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('m') => self.start_game(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

// fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
//     let [area] = Layout::horizontal([horizontal])
//         .flex(Flex::Center)
//         .areas(area);
//     let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
//     area
// }

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" idle game yass MENU ".bold());
        let instructions = Line::from(vec![
            " Create map ".into(),
            "<M> ".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        block.render(area, buf);
    }
}
