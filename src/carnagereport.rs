use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    symbols::border,
    text::Line,
    widgets::{Block, Clear},
};

use crate::{
    center_horizontal, center_vertical,
    upgrade::{PlayerState, PlayerStateDiff},
};

#[derive(Clone)]
pub struct CarnageReport {
    prev_player_state: PlayerState,
    new_player_state: PlayerState,
}

impl CarnageReport {
    pub fn new(prev_player_state: PlayerState, new_player_state: PlayerState) -> Self {
        return Self {
            prev_player_state,
            new_player_state,
        };
    }

    pub fn get_diff(&self) -> PlayerStateDiff {
        return self.new_player_state.clone() - self.prev_player_state.clone();
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = popup_area(frame.area(), 50, 30);

        let popup = Block::bordered()
            .border_set(border::PLAIN)
            .title(" Carnage Report ")
            .title_bottom(Line::from(vec![" <ESC> Upgrades ".into()]))
            .title_alignment(ratatui::layout::Alignment::Center);

        let inner_area = center_vertical(center_horizontal(popup.inner(area), 10), 1);

        let state_diff = self.get_diff();

        let inner = Line::from(vec![
            "Gold: ".into(),
            state_diff.inventory.gold.to_string().into(),
        ]);

        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
        frame.render_widget(inner, inner_area);
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
