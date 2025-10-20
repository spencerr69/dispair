pub struct Position(usize, usize);

impl Position {
    pub fn get(&self) -> (usize, usize) {
        (self.0, self.1)
    }

    pub fn set(&mut self, x: usize, y: usize) {
        self.0 = x;
        self.1 = y;
    }
}

pub struct Character {
    position: Position,
}

impl Character {
    pub fn new() -> Self {
        Character {
            position: Position(0, 0),
        }
    }

    pub fn move_up(&mut self) {
        let (x, y) = self.position.get();
        self.position.set(x, y + 1);
    }
}
