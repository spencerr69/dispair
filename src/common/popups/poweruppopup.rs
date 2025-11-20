use std::{cell::RefCell, rc::Rc};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    symbols::border,
    text::{Line, ToLine},
    widgets::{Block, Clear, Paragraph},
};

use crate::common::{
    center_horizontal, center_vertical,
    popups::popup_area,
    powerup::{DynPowerup, PoweruppableWeapon},
};

pub struct PowerupPopup {
    powerup_choices: Vec<DynPowerup>,
}

impl PowerupPopup {
    pub fn new(current_weapons: RefCell<Rc<Vec<Box<dyn PoweruppableWeapon>>>>) -> Self {
        let mut choices = Vec::new();

        current_weapons.borrow().iter().for_each(|weapon| {
            let next_upgrade = weapon.get_next_upgrade();
            if let Some(next_upgrade) = next_upgrade {
                choices.push(next_upgrade);
            }
        });

        Self {
            powerup_choices: choices,
        }
    }

    pub fn render_choices(&mut self, frame: &mut Frame, rect: Rect) {
        let horizontal = Layout::horizontal(vec![
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ]);

        let [left_r, mid_r, right_r] = horizontal.areas(rect);

        let block = Block::default().border_set(border::ROUNDED);

        let left_upgrade = self.powerup_choices.pop();

        let mid_upgrade = self.powerup_choices.pop();

        let right_upgrade = self.powerup_choices.pop();

        for (upgrade, area) in [
            (left_upgrade, left_r),
            (mid_upgrade, mid_r),
            (right_upgrade, right_r),
        ] {
            let name;
            let r_desc;
            let amount_string;

            let mut paragraph = Paragraph::new("").block(block.clone());

            if let Some(upgrade) = upgrade {
                name = upgrade.get_name().to_string();
                let title = name.to_line();

                r_desc = upgrade.get_desc().to_string();
                let desc = r_desc.to_line();

                let curr_level = upgrade.get_current_level();
                let new_level = upgrade.get_new_level();
                amount_string = format!("Level {} -> {}", curr_level, new_level);
                let amount = amount_string.to_line();

                paragraph = Paragraph::new(vec![title, desc, amount]).block(block.clone());
            }

            frame.render_widget(paragraph, area);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = popup_area(frame.area(), 80, 60);

        let popup = Block::bordered()
            .border_set(border::PLAIN)
            .title(" Powerup Choice ")
            .title_bottom(Line::from(vec![" <ESC> Upgrades ".into()]))
            .title_alignment(ratatui::layout::Alignment::Center);

        let inner_area = center_vertical(center_horizontal(popup.inner(area), 10), 1);

        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
        self.render_choices(frame, inner_area);
    }
}
