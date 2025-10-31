use std::{collections::HashMap, path::Path};

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize, ser::Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub upgrades: CurrentUpgrades,
    pub inventory: Inventory,
    pub stats: Stats,
}

impl PlayerState {
    pub fn refresh(&mut self) {
        self.stats = Stats::default();

        //upgrades 1 PRESERVE
        //upgrade 11: PRESERVE::\conform
        if !self.upgrade_owned(&"11".to_string()) {
            self.stats.enemy_spawn_mult = 50.;
            self.stats.timer = 10;
        }

        //upgrade 12 grow
        if self.upgrade_owned(&"12".to_string()) {
            self.stats.size += 1;
        }

        //upgrade 13 become
        if self.upgrade_owned(&"13".to_string()) {
            self.stats.enemy_spawn_mult += 0.5;
        }

        //upgrades 2 STATS
        //upgrade 211 damage/flat_up
        if self.upgrade_owned(&"211".to_string()) {
            self.stats.damage_flat_boost += 1 * self.amount_owned(&"211".to_string()) as i32;
        }

        //upgrade 212 damage/mult_up
        if self.upgrade_owned(&"212".to_string()) {
            self.stats.damage_mult += 0.05 * self.amount_owned(&"212".to_string()) as f64;
        }

        //upgrade 221 health/flat_up
        if self.upgrade_owned(&"221".to_string()) {
            self.stats.base_health += 1 * self.amount_owned(&"221".to_string()) as i32;
        }

        //upgrade 222 health/mult_up
        if self.upgrade_owned(&"222".to_string()) {
            self.stats.health_mult += 0.05 * self.amount_owned(&"222".to_string()) as f64;
        }

        //cleanups
        self.stats.health = (self.stats.base_health as f64 * self.stats.health_mult).ceil() as i32;
    }

    pub fn amount_owned(&self, id: &String) -> u32 {
        self.upgrades.get(id).unwrap_or(&0).clone()
    }

    pub fn upgrade_owned(&self, id: &String) -> bool {
        self.upgrades.get(id).unwrap_or(&0).clone() > 0
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        let upgrade_tree = get_upgrade_tree().unwrap();

        let mut out = Self {
            inventory: Inventory::default(),
            stats: Stats::default(),
            upgrades: get_current_upgrades(upgrade_tree, HashMap::new()),
        };

        out.refresh();

        out
    }
}

pub type CurrentUpgrades = HashMap<String, u32>;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct UpgradeNode {
    title: String,
    description: String,
    id: String,
    cost: Option<u32>,
    children: Option<Vec<UpgradeNode>>,
    limit: u32,
    requires: Vec<String>,
}

impl UpgradeNode {
    pub fn has_children(&self) -> bool {
        self.children.is_some() && self.children.clone().unwrap().len() > 0
    }

    pub fn get_display_title(&self) -> String {
        if self.children.is_some() {
            ">".to_string() + self.title.as_str()
        } else {
            self.title.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Inventory {
    pub gold: u32,
}

impl Inventory {
    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stats {
    pub base_health: i32,
    pub health_mult: f64,

    pub health: i32,

    pub damage_mult: f64,
    pub damage_flat_boost: i32,

    pub attack_speed_mult: f64,
    pub movement_speed_mult: f64,

    pub enemy_spawn_mult: f64,
    pub enemy_move_mult: f64,

    pub size: i32,

    pub width: usize,
    pub height: usize,

    pub timer: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            base_health: 10,
            health_mult: 1.,

            health: 10,

            damage_mult: 1.,
            damage_flat_boost: 0,

            attack_speed_mult: 1.,
            movement_speed_mult: 1.,

            enemy_spawn_mult: 1.,
            enemy_move_mult: 1.,

            width: 20,
            height: 6,

            size: 0,

            timer: 60,
        }
    }
}

pub type UpgradeTree = Vec<UpgradeNode>;

pub fn get_upgrade_tree() -> Result<Vec<UpgradeNode>, serde_json::Error> {
    let get_file = std::fs::read_to_string(Path::new("src/upgrades.json"))
        .map_err(|_| serde_json::Error::custom("naurrr"))?;
    let upgrade_tree: UpgradeTree = serde_json::from_str(get_file.as_str())?;
    Ok(upgrade_tree)
}

pub fn get_current_upgrades(
    upgrade_tree: UpgradeTree,
    mut acc: CurrentUpgrades,
) -> CurrentUpgrades {
    upgrade_tree.into_iter().for_each(|node| {
        acc.insert(node.id, 0);
        if let Some(children) = node.children {
            acc = get_current_upgrades(children, acc.clone());
        }
    });

    acc
}

pub struct UpgradesMenu {
    pub player_state: PlayerState,
    root_upgrade_tree: UpgradeTree,
    pub upgrade_selection: ListState,
    pub close: bool,
    pub current_layer: UpgradeTree,
    pub history: Vec<usize>,
}

impl UpgradesMenu {
    pub fn new(player_state: PlayerState) -> Self {
        let upgrade_tree = get_upgrade_tree().unwrap();
        let mut menu = Self {
            player_state,
            root_upgrade_tree: upgrade_tree.clone(),
            current_layer: upgrade_tree,
            upgrade_selection: ListState::default(),
            close: false,
            history: Vec::new(),
        };

        menu.upgrade_selection.select_first();

        menu
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('w') => self.prev_selection(),
            KeyCode::Char('s') => self.next_selection(),
            KeyCode::Enter => {
                if let Some(current_node) = self.get_selected_node() {
                    if current_node.has_children() {
                        self.navigate_into_upgrade();
                        self.upgrade_selection.select_first();
                    } else {
                        self.buy_upgrade().unwrap_or(());
                    }
                }
            }

            KeyCode::Esc => {
                if self.history.len() > 0 {
                    self.go_back();
                } else {
                    self.close = true;
                }
            }
            _ => {}
        }
    }

    pub fn buy_upgrade(&mut self) -> Result<(), String> {
        if let Some(current_node) = self.get_selected_node() {
            if current_node.cost.is_some() {
                if self.player_state.amount_owned(&current_node.id) >= current_node.limit {
                    return Err("Upgrade already owned".to_string());
                } else if current_node.cost.unwrap() > self.player_state.inventory.gold {
                    return Err("Not enough money".to_string());
                }
                self.player_state.inventory.gold -= current_node.cost.unwrap();
                let upgrade_count = self.player_state.upgrades.get_mut(&current_node.id);
                if let Some(count) = upgrade_count {
                    *count += 1;
                } else {
                    self.player_state.upgrades.insert(current_node.id, 1);
                }
                Ok(())
            } else {
                Err("Upgrade is not purchaseable".to_string())
            }
        } else {
            Err("No upgrade selected".to_string())
        }
    }

    pub fn prev_selection(&mut self) {
        self.upgrade_selection.select_previous();
    }

    pub fn next_selection(&mut self) {
        self.upgrade_selection.select_next();
    }

    pub fn go_back(&mut self) {
        self.history.pop();
        self.current_layer = self.root_upgrade_tree.clone();
        for index in self.history.clone() {
            self.current_layer = self.current_layer[index].children.clone().unwrap();
        }
    }

    pub fn get_selected_node(&self) -> Option<UpgradeNode> {
        let selected_index = self.upgrade_selection.selected()?;
        if self.current_layer.len() > selected_index {
            Some(self.current_layer[selected_index].clone())
        } else {
            None
        }
    }

    pub fn navigate_into_upgrade(&mut self) -> Option<()> {
        let selected_index = self.upgrade_selection.selected()?;
        if let Some(ref children) = self.current_layer[selected_index].children {
            self.current_layer = children.clone();
            self.history.push(selected_index);
            return Some(());
        }
        None
    }

    pub fn node_to_list(
        upgrade_nodes: Vec<UpgradeNode>,
        player_state: PlayerState,
    ) -> Vec<ListItem<'static>> {
        upgrade_nodes
            .iter()
            .map(|node| {
                let have_required = node.requires.iter().fold(true, |acc, current| {
                    acc && player_state.amount_owned(&current) > 0
                });

                if node.limit > 0 && player_state.amount_owned(&node.id) >= node.limit {
                    Some(ListItem::from(
                        node.get_display_title()
                            .clone()
                            .bold()
                            .underlined()
                            .dark_gray(),
                    ))
                } else if have_required {
                    Some(ListItem::from(node.get_display_title().clone()))
                } else {
                    None
                }
            })
            .filter(|i| i.is_some())
            .map(|i| i.unwrap())
            .collect()
    }

    pub fn render_upgrades(&mut self, frame: &mut Frame) {
        let mut block = Block::bordered().border_set(border::THICK);
        let inner = block.inner(frame.area());

        let gold = self.player_state.inventory.gold;
        let current_layer = self.current_layer.clone();

        let text: Vec<ListItem> = Self::node_to_list(current_layer, self.player_state.clone());

        let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Fill(1)]);
        let [left, right] = horizontal.areas(inner);

        let title = Line::from(" dispair ".bold());
        let instructions = Line::from(vec![" gold: ".into(), gold.to_string().into()]);
        block = block
            .title(title.left_aligned())
            .title_bottom(instructions.right_aligned());

        let list = List::new(text)
            .highlight_style(Style::new().slow_blink().bold())
            .highlight_symbol(">");

        let current_upgrade = self.get_selected_node().unwrap_or(UpgradeNode::default());

        let upgrade_block = Block::bordered().border_set(border::ROUNDED);
        let upgrade_title = Line::from(current_upgrade.clone().title);
        let upgrade_desc = Line::from(current_upgrade.clone().description);
        let mut upgrade_cost = Line::from("");
        if current_upgrade.cost.is_some() {
            upgrade_cost = Line::from(format!("${}", current_upgrade.cost.unwrap_or(0)));
        } else if current_upgrade.has_children() {
            upgrade_cost = Line::from("> enter folder")
        }

        let mut upgrade_amount = Line::from("");
        if current_upgrade.limit > 1 {
            upgrade_amount = Line::from(format!(
                "You have: {}/{}",
                self.player_state.amount_owned(&current_upgrade.id),
                current_upgrade.limit
            ));
        }

        let upgrade_paragraph = Paragraph::new(vec![
            upgrade_title,
            "".into(),
            upgrade_desc,
            "".into(),
            upgrade_cost,
            "".into(),
            upgrade_amount,
        ])
        .block(upgrade_block)
        .centered()
        .wrap(Wrap { trim: false });

        frame.render_widget(block, frame.area());

        frame.render_widget(upgrade_paragraph, right);

        frame.render_stateful_widget(list, left, &mut self.upgrade_selection);
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

    #[test]
    fn current_upgrades_check() {
        let upgrade_tree = get_upgrade_tree().unwrap();
        let current_upgrades = get_current_upgrades(upgrade_tree, HashMap::new());
        println!("Current upgrades: {:?}", current_upgrades);
        assert!(!current_upgrades.is_empty());
    }
}
