use std::path::Path;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text, ToLine, ToText},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget},
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
    pub upgrade_selection: ListState,
}

impl UpgradesMenu {
    pub fn new(player_state: PlayerState) -> Self {
        let upgrade_tree = get_upgrade_tree().unwrap();
        Self {
            player_state,
            upgrade_tree,
            upgrade_selection: ListState::default(),
        }
    }

    pub fn get_text(&self) -> Vec<Text<'_>> {
        self.upgrade_tree
            .iter()
            .map(|node| node.title.to_text())
            .collect()
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('s') => self.next_selection(),
            _ => {}
        }
    }

    pub fn next_selection(&mut self) {
        self.upgrade_selection.select_next();
    }

    pub fn get_info(&self) -> Option<usize> {
        self.upgrade_selection.selected()
    }

    pub fn node_to_list(upgrade_nodes: &Vec<UpgradeNode>) -> Vec<ListItem<'_>> {
        upgrade_nodes
            .iter()
            .map(|node| ListItem::from(node.title.to_text()))
            .collect()
    }

    pub fn render_upgrades(&mut self, frame: &mut Frame) {
        let title = Line::from(" spattui ".bold());

        let instructions = Line::from(vec![" health: ".into(), " ".into()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        let text: Vec<ListItem> = Self::node_to_list(&self.upgrade_tree);
        let list = List::new(text)
            .block(block)
            .highlight_style(Style::new().slow_blink().bold())
            .highlight_symbol(">");
        frame.render_stateful_widget(list, frame.area(), &mut self.upgrade_selection);
    }
}

// impl StatefulWidget for &UpgradesMenu {
//     type State = ListState;
//     fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         let title = Line::from(" spattui ".bold());

//         let instructions = Line::from(vec![" health: ".into(), " ".into()]);
//         let block = Block::bordered()
//             .title(title.centered())
//             .title_bottom(instructions.centered())
//             .border_set(border::THICK);

//         List::new(self.get_text())
//             .block(block)
//             .highlight_style(Style::new().slow_blink().bold())
//             .highlight_symbol(">")
//             .render(area, buf, state);
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correctly() {
        let upgrade_tree = get_upgrade_tree().unwrap();
        assert_eq!(upgrade_tree[0].title, "PRESERVE")
    }
}
