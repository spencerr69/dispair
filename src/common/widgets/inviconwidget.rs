use crate::common::powerup::Poweruppable;
use crate::common::utils::center;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Widget};

#[derive(Clone)]
pub struct InvIconWidget<'a> {
    pub source: &'a dyn Poweruppable,
    pub level: i32,
    pub title: String,
}

impl<'a> InvIconWidget<'a> {
    pub fn new(source: &'a dyn Poweruppable) -> Self {
        Self {
            source,
            level: source.get_level(),
            title: source.get_name().clone(),
        }
    }
}

impl Widget for InvIconWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(self.level.to_string())
            .title_alignment(Alignment::Center)
            .light_blue();
        let inner_area = block.inner(area);
        let centered = center(inner_area, 1, 1);
        let inner_char = self.title.get(..1).unwrap_or(" ");

        let inner_text = Text::raw(inner_char);

        block.render(area, buf);
        inner_text.render(centered, buf);
    }
}
