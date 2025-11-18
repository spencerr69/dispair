//! This module defines coordinate-related structs and enums, such as `Position`, `Area`, and `Direction`.
//! It provides functionality for working with positions and areas within the game world.
use crate::common::roguegame::Layer;

/// Represents a 2D position with x and y coordinates.
#[derive(Clone, Default, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct Position(pub i32, pub i32);

impl Position {
    /// Creates a new `Position`, ensuring that coordinates are not negative.
    pub fn new(x: i32, y: i32) -> Self {
        let new_x: i32;
        let new_y: i32;
        if x < 0 {
            new_x = 0;
        } else {
            new_x = x;
        }
        if y < 0 {
            new_y = 0;
        } else {
            new_y = y;
        }

        Position(new_x, new_y)
    }

    /// Returns the (x, y) coordinates of the position.
    pub fn get(&self) -> (i32, i32) {
        (self.0, self.1)
    }

    /// Returns the (x, y) coordinates as `usize`.
    pub fn get_as_usize(&self) -> (usize, usize) {
        (self.0.max(0) as usize, self.1.max(0) as usize)
    }

    /// Constrains the position to be within the boundaries of the given layer.
    pub fn constrain(&mut self, layer: &Layer) {
        self.0 = self.0.max(0).min(layer[0].len() as i32 - 1);
        self.1 = self.1.max(0).min(layer.len() as i32 - 1);
    }

    /// Calculates the (dx, dy) distance between two positions.
    pub fn get_distance(&self, other: &Position) -> (i32, i32) {
        let (self_x, self_y) = self.get();
        let (other_x, other_y) = other.get();
        (other_x - self_x, other_y - self_y)
    }

    /// Checks if the position is within the given area.
    pub fn is_in_area(&self, area: &Area) -> bool {
        let (x, y) = self.get();
        let (min_x, min_y, max_x, max_y) = area.get_bounds();
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }
}

/// Represents the four cardinal directions.
#[derive(Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

/// Represents a rectangular area defined by two corner positions.
#[derive(Clone)]
pub struct Area {
    pub corner1: Position,
    pub corner2: Position,
}

impl From<Layer> for Area {
    fn from(value: Layer) -> Self {
        Self {
            corner1: Position(0, 0),
            corner2: Position(value[0].len() as i32, value.len() as i32),
        }
    }
}

impl From<Position> for Area {
    fn from(value: Position) -> Self {
        Self {
            corner1: value.clone(),
            corner2: value,
        }
    }
}

impl Area {
    /// Constructs an Area defined by two corner positions.
    ///
    /// The provided positions become the area's corners; the effective bounds are computed from them when needed.
    ///
    /// # Examples
    ///
    /// ```
    /// let a = Area::new(Position::new(1, 2), Position::new(3, 4));
    /// let (min_x, min_y, max_x, max_y) = a.get_bounds();
    /// assert_eq!((min_x, min_y, max_x, max_y), (1, 2, 3, 4));
    /// ```
    pub fn new(corner1: Position, corner2: Position) -> Self {
        Area { corner1, corner2 }
    }

    /// Constructs an Area with both corners at the world origin (0, 0).
    ///
    /// # Examples
    ///
    /// ```
    /// let a = Area::origin();
    /// assert_eq!(a.get_bounds(), (0, 0, 0, 0));
    /// ```
    pub fn origin() -> Area {
        Area {
            corner1: Position(0, 0),
            corner2: Position(0, 0),
        }
    }

    /// Compute the axis-aligned bounding box that encloses the area's corners.
    ///
    /// The returned tuple is (min_x, min_y, max_x, max_y).
    ///
    /// # Examples
    ///
    /// ```
    /// let a = Area::new(Position::new(3, 5), Position::new(1, 2));
    /// assert_eq!(a.get_bounds(), (1, 2, 3, 5));
    /// ```
    pub fn get_bounds(&self) -> (i32, i32, i32, i32) {
        let (x1, y1) = self.corner1.get();
        let (x2, y2) = self.corner2.get();

        (x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
    }

    /// Constrains the area to be within the boundaries of the given layer.
    pub fn constrain(&mut self, layer: &Layer) {
        self.corner1.constrain(layer);
        self.corner2.constrain(layer);
    }
}

impl IntoIterator for Area {
    type Item = Position;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        let (x1, y1, x2, y2) = self.get_bounds();
        Box::new((x1..=x2).flat_map(move |x| (y1..=y2).map(move |y| Position(x, y))))
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
        assert_eq!(position.get(), (3, 0))
    }

    #[test]
    fn area_iter() {
        let area = Area::new(Position(3, 2), Position(6, 5));

        assert_eq!(area.clone().into_iter().fold(0, |acc, _| acc + 1), 16);
        assert_eq!(area.clone().into_iter().max(), Some(Position(6, 5)));
    }
}