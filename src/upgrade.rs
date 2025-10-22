use std::path::Path;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, ToLine},
    widgets::{Block, Paragraph, Widget},
};
use serde::{Deserialize, Serialize, ser::Error};

#[derive(Debug, Default, Clone)]
pub struct PlayerState {
    upgrades: Vec<UpgradeNode>,
    inventory: Inventory,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpgradeNode {
    title: String,
    description: String,
    id: String,
    value: Option<f64>,
    cost: Option<i64>,
    children: Option<Vec<UpgradeNode>>,
}

#[derive(Default, Debug, Clone)]
pub struct Inventory {
    gold: i32,
}
pub type UpgradeTree = Vec<UpgradeNode>;

pub fn get_upgrade_tree() -> Result<Vec<UpgradeNode>, serde_json::Error> {
    let get_file = std::fs::read_to_string(Path::new("src/upgrades.json"))
        .map_err(|_| serde_json::Error::custom("naurrr"))?;
    let upgrade_tree: UpgradeTree = serde_json::from_str(get_file.as_str())?;
    Ok(upgrade_tree)
}

pub struct UpgradesMenu {
    player_state: PlayerState,
    upgrade_tree: UpgradeTree,
}

impl UpgradesMenu {
    pub fn new(player_state: PlayerState) -> Self {
        let upgrade_tree = get_upgrade_tree().unwrap();
        Self {
            player_state,
            upgrade_tree,
        }
    }

    pub fn get_text(&self) -> Vec<Line<'_>> {
        self.upgrade_tree
            .iter()
            .map(|node| node.title.to_line())
            .collect()
    }
}

impl Widget for &UpgradesMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" spattui ".bold());

        let instructions = Line::from(vec![" health: ".into(), " ".into()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Paragraph::new(self.get_text())
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correctly() {
        let upgrade_tree = get_upgrade_tree().unwrap();
        assert_eq!(upgrade_tree[0].title, "PRESERVE")
    }
}
