//! This module defines coordinate-related structs and enums, such as `Position`, `Area`, and `Direction`.
//! It provides functionality for working with positions and areas within the game world.
use std::{cell::RefCell, rc::Rc};

use crate::common::roguegame::Layer;

/// Represents a 2D position with x and y coordinates.
#[derive(Clone, Default, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Position(pub i32, pub i32);

impl Position {
    /// Creates a new `Position`, ensuring that coordinates are not negative.
    #[must_use]
    pub fn new(x: i32, y: i32) -> Self {
        let new_x = if x < 0 { 0 } else { x };
        let new_y = if y < 0 { 0 } else { y };

        Position(new_x, new_y)
    }

    /// Returns the (x, y) coordinates of the position.
    #[must_use]
    pub fn get(&self) -> (i32, i32) {
        (self.0, self.1)
    }

    /// Returns the (x, y) coordinates as `usize`.
    #[must_use]
    pub fn get_as_usize(&self) -> (usize, usize) {
        (self.0.max(0) as usize, self.1.max(0) as usize)
    }

    /// Constrains the position to be within the boundaries of the given layer.
    pub fn constrain(&mut self, layer: &Layer) {
        self.0 = self.0.max(0).min(layer[0].len() as i32 - 1);
        self.1 = self.1.max(0).min(layer.len() as i32 - 1);
    }

    /// Calculates the (dx, dy) distance between two positions.
    #[must_use]
    pub fn get_distance(&self, other: &Position) -> (i32, i32) {
        let (self_x, self_y) = self.get();
        let (other_x, other_y) = other.get();
        (other_x - self_x, other_y - self_y)
    }

    /// Checks if the position is within the given area.
    pub fn is_in_area(&self, area: &Rc<RefCell<dyn Area>>) -> bool {
        let (x, y) = self.get();
        let (min_x, min_y, max_x, max_y) = area.borrow().get_bounds();
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }
}

/// Represents the four cardinal directions.
#[derive(Clone, PartialEq, Eq)]
pub enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

/// Represents a rectangular area defined by two corner positions.
#[derive(Clone)]
pub struct SquareArea {
    pub corner1: Position,
    pub corner2: Position,
}

pub trait Area {
    fn get_positions(&self) -> Vec<Position>;

    fn pos_iter(&self) -> Box<dyn Iterator<Item = Position>>;

    fn get_bounds(&self) -> (i32, i32, i32, i32);

    fn constrain(&mut self, layer: &Layer);
}

impl From<Layer> for SquareArea {
    fn from(value: Layer) -> Self {
        Self {
            corner1: Position(0, 0),
            corner2: Position(value[0].len() as i32, value.len() as i32),
        }
    }
}

impl From<Position> for SquareArea {
    fn from(value: Position) -> Self {
        Self {
            corner1: value.clone(),
            corner2: value,
        }
    }
}

impl SquareArea {
    /// Constructs an Area defined by two corner positions.
    ///
    /// The provided positions become the area's corners; the effective bounds are computed from them when needed.
    #[must_use]
    pub fn new(corner1: Position, corner2: Position) -> Self {
        SquareArea { corner1, corner2 }
    }

    /// Constructs an Area with both corners at the world origin (0, 0).\n
    #[must_use]
    pub fn origin() -> SquareArea {
        SquareArea {
            corner1: Position(0, 0),
            corner2: Position(0, 0),
        }
    }
}

impl Area for SquareArea {
    fn get_positions(&self) -> Vec<Position> {
        self.pos_iter().collect()
    }

    fn pos_iter(&self) -> Box<dyn Iterator<Item = Position>> {
        let (x1, y1, x2, y2) = self.get_bounds();
        Box::new((x1..=x2).flat_map(move |x| (y1..=y2).map(move |y| Position(x, y))))
    }

    /// Compute the axis-aligned bounding box that encloses the area's corners.
    ///
    /// The returned tuple is `(min_x, min_y, max_x, max_y)`.
    fn get_bounds(&self) -> (i32, i32, i32, i32) {
        let (x1, y1) = self.corner1.get();
        let (x2, y2) = self.corner2.get();

        (x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
    }

    fn constrain(&mut self, layer: &Layer) {
        self.corner1.constrain(layer);
        self.corner2.constrain(layer);
    }
}

#[derive(Clone)]
pub struct ChaosArea {
    pub position_list: Vec<Position>,
}

impl ChaosArea {
    #[must_use]
    pub fn new(position_list: Vec<Position>) -> Self {
        ChaosArea { position_list }
    }
}

impl Area for ChaosArea {
    fn get_positions(&self) -> Vec<Position> {
        self.position_list.clone()
    }

    fn pos_iter(&self) -> Box<dyn Iterator<Item = Position>> {
        Box::new(self.position_list.clone().into_iter())
    }

    fn get_bounds(&self) -> (i32, i32, i32, i32) {
        self.pos_iter()
            .fold((i32::MAX, i32::MAX, i32::MIN, i32::MIN), |acc, item| {
                let (x, y) = item.get();
                (acc.0.min(x), acc.1.min(y), acc.2.max(x), acc.3.max(y))
            })
    }

    fn constrain(&mut self, layer: &Layer) {
        self.position_list
            .iter_mut()
            .for_each(|pos| pos.constrain(layer));
    }
}

#[cfg(test)]
mod tests {
    use crate::common::roguegame::EntityCharacters;

    use super::*;

    #[test]
    fn position_above_0() {
        let result = Position::new(4, 4);
        assert_eq!(result.get(), (4, 4));
    }

    #[test]
    fn position_of_0() {
        let result = Position::new(0, 0);
        assert_eq!(result.get(), (0, 0));
    }

    #[test]
    fn position_below_0() {
        let result = Position::new(-1, -1);
        assert_eq!(result.get(), (0, 0));
    }

    #[test]
    fn position_constrain() {
        let mut position = Position(50, 50);
        let layer: Layer = Vec::from([Vec::from([
            EntityCharacters::Empty,
            EntityCharacters::Empty,
            EntityCharacters::Empty,
            EntityCharacters::Empty,
        ])]);
        position.constrain(&layer);
        assert_eq!(position.get(), (3, 0));
    }

    #[test]
    fn area_iter() {
        let area = SquareArea::new(Position(3, 2), Position(6, 5));

        assert_eq!(area.clone().pos_iter().fold(0, |acc, _| acc + 1), 16);
        assert_eq!(area.clone().pos_iter().max(), Some(Position(6, 5)));
    }
}
