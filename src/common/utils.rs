use crate::common::TICK_RATE;
use crate::common::character::{Character, Movable, Renderable};
use crate::common::coords::{Direction, Position};
use crate::common::map::Layer;
use rand::Rng;
use ratatui::layout::{Constraint, Layout, Rect};
use std::fmt::format;

/// Centers a `Rect` vertically within a given area.
#[must_use]
pub fn center_vertical(area: Rect, height: u16) -> Rect {
    let [centered_area] = Layout::vertical([Constraint::Length(height)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

/// Centers a `Rect` horizontally within a given area.
#[must_use]
pub fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [centered_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    centered_area
}

/// Centers a `Rect` both vertically and horizontally within a given area.
#[must_use]
pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let centered_area = center_vertical(area, height);
    center_horizontal(centered_area, width)
}

#[must_use]
pub fn get_rand_position_on_edge(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let which_edge = rng.random_range(0..4);

    match which_edge {
        0 => Position::new(0, rng.random_range(0..layer.len() as i32)),
        1 => Position::new(
            layer[0].len() as i32 - 1,
            rng.random_range(0..layer.len() as i32),
        ),
        2 => Position::new(rng.random_range(0..layer[0].len() as i32), 0),
        3 => Position::new(
            rng.random_range(0..layer[0].len() as i32),
            layer.len() as i32 - 1,
        ),
        _ => Position::new(0, 0),
    }
}

#[must_use]
pub fn get_rand_position_on_layer(layer: &Layer) -> Position {
    let mut rng = rand::rng();

    let x = rng.random_range(0..layer[0].len() as i32);
    let y = rng.random_range(0..layer.len() as i32);
    Position::new(x, y)
}

#[must_use]
pub fn is_next_to_character(char_position: &Position, position: &Position) -> bool {
    let (x, y) = position.get_as_usize();
    let (char_x, char_y) = char_position.get_as_usize();

    (x == char_x.saturating_add(1) || x == char_x.saturating_sub(1)) && y == char_y
        || (y == char_y.saturating_add(1) || y == char_y.saturating_sub(1)) && x == char_x
}

#[must_use]
pub fn can_stand(
    width: i32,
    height: i32,
    character: Option<&Character>,
    position: &Position,
) -> bool {
    let (x, y) = position.get();

    let char_collision = character.map(|c| c.get_pos() == position).unwrap_or(false);

    if x < 0 || x >= width || y < 0 || y >= height || char_collision {
        return false;
    }
    true
}

pub fn move_entity(layer: &mut Layer, entity: &mut impl Movable, direction: Direction) {
    let (x, y) = entity.get_pos().get();
    let mut new_pos = match direction {
        Direction::LEFT => Position::new(x - 1, y),
        Direction::RIGHT => Position::new(x + 1, y),
        Direction::UP => Position::new(x, y - 1),
        Direction::DOWN => Position::new(x, y + 1),
    };

    new_pos.constrain(layer);

    if can_stand(layer[0].len() as i32, layer.len() as i32, None, &new_pos) {
        entity.move_to(new_pos, direction);
        // update_entity_positions(layer, entity);
    } else {
        entity.move_to(entity.get_pos().clone(), direction);
    }
}

pub fn get_mut_item_in_2d_enum_vec<'a, T>(
    vec: &'a mut [(usize, Vec<(usize, T)>)],
    position: &'a Position,
) -> Option<&'a mut T> {
    let (x, y) = position.get_as_usize();
    let maybe_row = vec.iter_mut().find(|(in_y, _)| in_y == &y);
    if let Some(row) = maybe_row {
        let maybe_item = row.1.iter_mut().find(|(in_x, _)| in_x == &x);
        if let Some(item) = maybe_item {
            Some(&mut item.1)
        } else {
            None
        }
    } else {
        None
    }
}

#[must_use]
pub fn per_sec_to_tick_count_to_u64(per_sec: f64) -> u64 {
    let per_tick = TICK_RATE / per_sec;
    per_tick.ceil() as u64
}
#[must_use]
pub fn per_sec_to_tick_count(per_sec: f64) -> f64 {
    TICK_RATE / per_sec
}

pub fn trim_string(s: String, max_len: usize) -> String {
    if s.len() > max_len {
        s[0..max_len].to_string()
    } else {
        s
    }
}

pub fn convert_range(value: f64, old_min: f64, old_max: f64, new_min: f64, new_max: f64) -> f64 {
    let old_range = old_max - old_min;
    let new_range = new_max - new_min;
    (((value - old_min) * new_range) / old_range) + new_min
}
