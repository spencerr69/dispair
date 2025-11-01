use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    symbols::{self, border},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::upgrade::{PlayerState, UpgradeNode, UpgradeTree, get_upgrade_tree};

#[derive(Clone)]
pub enum Goto {
    Game,
    Menu,
}

pub struct UpgradesMenu {
    pub player_state: PlayerState,
    root_upgrade_tree: UpgradeTree,
    pub upgrade_selection: ListState,
    pub close: Option<Goto>,
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
            close: None,
            history: Vec::new(),
        };

        menu.upgrade_selection.select_first();

        menu
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('w') | KeyCode::Up => self.prev_selection(),
            KeyCode::Char('s') | KeyCode::Down => self.next_selection(),
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
            KeyCode::Char(' ') => self.close = Some(Goto::Game),

            KeyCode::Esc => {
                if self.history.len() > 0 {
                    self.go_back();
                } else {
                    self.close = Some(Goto::Menu);
                }
            }
            _ => {}
        }
    }

    pub fn buy_upgrade(&mut self) -> Result<(), String> {
        if let Some(current_node) = self.get_selected_node() {
            if current_node.cost.is_some() {
                let next_cost =
                    current_node.next_cost(self.player_state.amount_owned(&current_node.id));

                if self.player_state.amount_owned(&current_node.id) >= current_node.limit {
                    return Err("Upgrade already owned".to_string());
                } else if next_cost > self.player_state.inventory.gold {
                    return Err("Not enough money".to_string());
                }
                self.player_state.inventory.gold -= next_cost;
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
                        node.get_display_title().clone().bold().italic().dark_gray(),
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

        let horizontal = Layout::horizontal([Constraint::Percentage(70), Constraint::Fill(1)]);
        let [left, right] = horizontal.areas(inner);

        let title = Line::from(" dispair ".bold());
        let gold_amount = Line::from(vec![" gold: ".into(), gold.to_string().into()]);
        let instructions = Line::from(vec![
            " <W|UP> Up | <S|DOWN> Down | <SPACE> Start Game | <Esc> Back ".into(),
        ]);
        block = block
            .title(title.left_aligned())
            .title_bottom(instructions.left_aligned());

        let list = List::new(text)
            .highlight_style(Style::new().rapid_blink().bold())
            .highlight_symbol(">");

        let current_upgrade = self.get_selected_node().unwrap_or(UpgradeNode::default());

        let upgrade_block = Block::bordered().border_set(border::ROUNDED);
        let upgrade_title = Line::from(current_upgrade.clone().title);
        let upgrade_desc = Line::from(current_upgrade.clone().description);
        let mut upgrade_cost = Line::from("");
        if current_upgrade.cost.is_some() {
            upgrade_cost = Line::from(format!(
                "${}",
                current_upgrade.next_cost(self.player_state.amount_owned(&current_upgrade.id))
            ));
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
        .block(upgrade_block.title_bottom(gold_amount.centered()))
        .centered()
        .wrap(Wrap { trim: false });

        frame.render_widget(block, frame.area());

        frame.render_widget(upgrade_paragraph, right);

        frame.render_stateful_widget(list, left, &mut self.upgrade_selection);
    }
}
