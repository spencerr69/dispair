use ratatui::layout::{Constraint, Layout, Rect};

pub mod carnagereport;
pub mod character;
pub mod coords;
pub mod effects;
pub mod enemy;
pub mod roguegame;
pub mod timescaler;
pub mod upgrade;
pub mod upgrademenu;
pub mod weapon;

pub fn center_vertical(area: Rect, height: u16) -> Rect {
    let [centered_area] = Layout::vertical([Constraint::Length(height)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

pub fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [centered_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let centered_area = center_vertical(area, height);
    center_horizontal(centered_area, width)
}

pub const TICK_RATE: f64 = 30.0;
pub const FRAME_RATE: f64 = 180.0;
