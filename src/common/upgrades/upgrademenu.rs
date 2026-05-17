//! This module provides the UI and logic for the upgrade menu.
//! It allows the player to navigate and purchase upgrades for their character.

use crate::common::upgrades::upgrade::{PlayerState, UpgradeNode, UpgradeTree, get_upgrade_tree};
use crate::common::{Goto, Viewable};
use crate::prelude::{KeyCode, KeyEvent};
use ratatui::text::{Span, Text};
use ratatui::widgets::BorderType;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct MenuCrumb {
    pub index: usize,
    pub title: String,
}

pub type MenuHistory = Vec<MenuCrumb>;

/// A struct that manages the state and rendering of the upgrades menu.
pub struct UpgradesMenu {
    pub player_state: Rc<RefCell<PlayerState>>,
    root_upgrade_tree: UpgradeTree,
    pub upgrade_selection: ListState,
    pub goto: Goto,
    pub current_layer: UpgradeTree,
    history: MenuHistory,
}

impl UpgradesMenu {
    /// Creates a new `UpgradesMenu` instance.
    ///
    /// # Panics
    ///
    /// Will panic if the upgrade tree cannot be retrieved.
    #[must_use]
    pub fn new(player_state: Rc<RefCell<PlayerState>>) -> Self {
        let upgrade_tree = get_upgrade_tree().unwrap();
        let mut menu = Self {
            player_state,
            root_upgrade_tree: upgrade_tree.clone(),
            current_layer: upgrade_tree,
            upgrade_selection: ListState::default(),
            goto: Goto::Upgrades,
            history: Vec::new(),
        };

        menu.upgrade_selection.select_first();

        menu
    }

    /// Handles key events for the upgrade menu.
    pub fn key_event(&mut self, key_event: &KeyEvent) {
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
            KeyCode::Char(' ') => self.goto = Goto::Game,

            KeyCode::Esc => {
                if !self.history.is_empty() {
                    self.go_back();
                    self.upgrade_selection.select_first();
                } else {
                    self.goto = Goto::Menu;
                }
            }
            _ => {}
        }
    }

    /// Attempts to buy the currently selected upgrade.
    ///
    /// # Errors
    ///
    /// Will return a `String` error if the user doesn't have enough gold or other issues are found
    pub fn buy_upgrade(&mut self) -> Result<(), String> {
        if let Some(current_node) = self.get_selected_node() {
            if current_node.cost.is_some() {
                let next_cost = current_node
                    .next_cost(self.player_state.borrow().amount_owned(&current_node.id));

                if self.player_state.borrow().amount_owned(&current_node.id) >= current_node.limit {
                    return Err("Upgrade already owned".to_string());
                } else if u128::from(next_cost) > self.player_state.borrow().inventory.gold {
                    return Err("Not enough money".to_string());
                }

                let mut player_state_mut = self.player_state.borrow_mut();

                player_state_mut.inventory.gold -= u128::from(next_cost);
                let upgrade_count = player_state_mut.upgrades.get_mut(&current_node.id);
                if let Some(count) = upgrade_count {
                    *count += 1;
                } else {
                    player_state_mut.upgrades.insert(current_node.id, 1);
                }
                Ok(())
            } else {
                Err("Upgrade is not purchasable".to_string())
            }
        } else {
            Err("No upgrade selected".to_string())
        }
    }

    /// Selects the previous item in the upgrade list.
    pub fn prev_selection(&mut self) {
        self.upgrade_selection.select_previous();
    }

    /// Selects the next item in the upgrade list.
    pub fn next_selection(&mut self) {
        self.upgrade_selection.select_next();
    }

    /// Navigates back to the previous layer in the upgrade tree.
    #[allow(clippy::missing_panics_doc)]
    pub fn go_back(&mut self) {
        self.history.pop();
        self.current_layer = self.root_upgrade_tree.clone();
        for crumb in self.history.clone() {
            self.current_layer = self.current_layer[crumb.index].children.clone().unwrap();
        }
    }

    /// Returns the currently selected `UpgradeNode`.
    #[must_use]
    pub fn get_selected_node(&self) -> Option<UpgradeNode> {
        let selected_index = self.upgrade_selection.selected()?;
        if self.current_layer.len() > selected_index {
            Some(self.current_layer[selected_index].clone())
        } else {
            None
        }
    }

    /// Navigates into the children of the currently selected upgrade node.
    pub fn navigate_into_upgrade(&mut self) -> Option<()> {
        let selected_index = self.upgrade_selection.selected()?;
        if let Some(ref children) = self.current_layer[selected_index].children {
            let menu_crumb = MenuCrumb {
                title: self.current_layer[selected_index].get_raw_title().clone(),
                index: selected_index,
            };
            self.history.push(menu_crumb);
            self.current_layer = children.clone();
            return Some(());
        }
        None
    }

    /// Converts a vector of `UpgradeNode`s to a vector of `ListItem`s for rendering.
    #[must_use]
    pub fn node_to_list(
        upgrade_nodes: &[UpgradeNode],
        player_state: &PlayerState,
    ) -> Vec<ListItem<'static>> {
        upgrade_nodes
            .iter()
            .filter_map(|node| {
                let have_required = node
                    .requires
                    .iter()
                    .all(|current| player_state.amount_owned(current) > 0);

                if node.limit == 0 && Self::own_children(node.clone(), player_state)
                    || (node.limit > 0 && player_state.amount_owned(&node.id) >= node.limit)
                {
                    Some(ListItem::from(node.get_display_title().clone().dark_gray()))
                } else if have_required {
                    Some(ListItem::from(node.get_display_title().clone().white()))
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub fn breadcrumbs_to_text(breadcrumbs: &'_ MenuHistory) -> Line<'_> {
        let mut history = vec![Span::raw("upgrades").dark_gray()];
        for crumb in breadcrumbs {
            history.push(Span::raw(" > ").dark_gray());
            history.push(Span::raw(&crumb.title).dark_gray());
        }
        if let Some(current_crumb) = history.last_mut() {
            *current_crumb = current_crumb.clone().bold().white();
        }
        Line::from(history)
    }

    /// Recursively checks if all children of an upgrade node are owned.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn own_children(upgrade_node: UpgradeNode, player_state: &PlayerState) -> bool {
        let have_required = upgrade_node
            .requires
            .iter()
            .all(|current| player_state.amount_owned(current) > 0);

        if let Some(children) = upgrade_node.children {
            for child in children {
                if !Self::own_children(child, player_state) {
                    return false;
                }
            }
            true
        } else {
            if !have_required {
                return true;
            }
            player_state.amount_owned(&upgrade_node.id) >= upgrade_node.limit
        }
    }

    /// Renders the upgrades menu to the frame.
    pub fn render_upgrades(&mut self, frame: &mut Frame) {
        let mut window = Block::bordered().border_set(border::THICK);
        let inner = window.inner(frame.area());

        let player_state = self.player_state.borrow();

        let gold = player_state.inventory.gold;
        let current_layer = self.current_layer.clone();

        let list_text: Vec<ListItem> = Self::node_to_list(&current_layer, &player_state);

        let horizontal = Layout::horizontal([Constraint::Percentage(70), Constraint::Fill(1)]);
        let [left, right] = horizontal.areas(inner);

        let title = Line::from(" dispair.upgrade ".bold());
        let gold_amount = Line::from(vec![" Gold: ".into(), gold.to_string().into()]);
        let instructions = Line::from(vec![
            " <W|UP> Up | <S|DOWN> Down | <SPACE> Start Game | <Esc> Back ".into(),
        ]);
        window = window
            .title(title.left_aligned())
            .title_bottom(instructions.left_aligned());

        let breadcrumbs_border = Block::bordered().border_type(BorderType::Rounded);

        let [list_rect, breadcrumb_rect] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(left);

        let breadcrumbs_inner = breadcrumbs_border.inner(breadcrumb_rect);

        let breadcrumbs = Self::breadcrumbs_to_text(&self.history);

        let list = List::new(list_text)
            .highlight_style(Style::new().bold())
            .highlight_symbol(">");

        let current_upgrade = self.get_selected_node().unwrap_or_default();

        let upgrade_block = Block::bordered().border_set(border::ROUNDED);
        let upgrade_title = Line::from(current_upgrade.clone().get_raw_title());
        let upgrade_desc = Text::from(current_upgrade.clone().description);
        let mut upgrade_cost = Line::from("");
        if current_upgrade.limit > 0
            && player_state.amount_owned(&current_upgrade.id) >= current_upgrade.limit
        {
            upgrade_cost = Line::from("owned");
        } else if current_upgrade.cost.is_some() {
            upgrade_cost = Line::from(format!(
                "${}",
                current_upgrade.next_cost(player_state.amount_owned(&current_upgrade.id))
            ));
        } else if current_upgrade.has_children() {
            upgrade_cost = Line::from("> enter folder");
        }

        let mut upgrade_amount = Line::from("");
        if current_upgrade.limit > 1 {
            upgrade_amount = Line::from(format!(
                "You have: {}/{}",
                player_state.amount_owned(&current_upgrade.id),
                current_upgrade.limit
            ));
        }

        let mut upgrade_lines = Vec::new();
        upgrade_lines.push(upgrade_title);
        upgrade_lines.push("".into());
        upgrade_lines.append(&mut upgrade_desc.lines.clone());
        upgrade_lines.push("".into());
        upgrade_lines.push(upgrade_cost);
        upgrade_lines.push("".into());
        upgrade_lines.push(upgrade_amount);

        let upgrade_paragraph = Paragraph::new(upgrade_lines)
            .block(upgrade_block.title_bottom(gold_amount.centered()))
            .centered()
            .wrap(Wrap { trim: false });

        frame.render_widget(window, frame.area());

        frame.render_widget(upgrade_paragraph, right);

        frame.render_stateful_widget(list, list_rect, &mut self.upgrade_selection);
        frame.render_widget(breadcrumbs_border, breadcrumb_rect);
        frame.render_widget(breadcrumbs, breadcrumbs_inner);
    }
}

impl Viewable for UpgradesMenu {
    fn tick(&mut self) {}

    fn get_goto(&self) -> &Goto {
        &self.goto
    }

    fn render(&mut self, frame: &mut Frame) {
        self.render_upgrades(frame)
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        self.key_event(key_event);
    }
}
