use ratatui::{
    Frame,
    symbols::border,
    text::Line,
    widgets::{Block, Clear},
};

use crate::common::{
    center_horizontal, center_vertical,
    popups::popup_area,
    upgrades::upgrade::{PlayerState, PlayerStateDiff},
};

/// Displays the results of a game level to the player.
#[derive(Clone)]
pub struct CarnageReport {
    prev_player_state: PlayerState,
    new_player_state: PlayerState,
}

impl CarnageReport {
    /// Creates a new `CarnageReport`.
    #[must_use]
    pub fn new(prev_player_state: PlayerState, new_player_state: PlayerState) -> Self {
        Self {
            prev_player_state,
            new_player_state,
        }
    }

    /// Calculates the difference between the player's state before and after the level.
    #[must_use]
    pub fn get_diff(&self) -> PlayerStateDiff {
        self.new_player_state.clone() - self.prev_player_state.clone()
    }

    /// Renders the carnage report to the screen.
    pub fn render(&self, frame: &mut Frame) {
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
