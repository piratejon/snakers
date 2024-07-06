use crate::coord::Coord;

// specific values so we can use as array indices
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

const ROTATE_UP: ((i32, i32), (i32, i32)) = ((1, 0), (0, 1));
const ROTATE_LEFT: ((i32, i32), (i32, i32)) = ((0, 1), (-1, 0));
const ROTATE_DOWN: ((i32, i32), (i32, i32)) = ((-1, 0), (0, -1));
const ROTATE_RIGHT: ((i32, i32), (i32, i32)) = ((0, -1), (1, 0));

impl Direction {
    pub fn rotation_matrix(&self) -> &((i32, i32), (i32, i32)) {
        match self {
            &Direction::Up => &ROTATE_UP,
            &Direction::Right => &ROTATE_RIGHT,
            &Direction::Down => &ROTATE_DOWN,
            &Direction::Left => &ROTATE_LEFT,
        }
    }

    pub fn rotate(&self, p: &Coord) -> Coord {
        let rot = self.rotation_matrix();
        return Coord {
            x: (p.x * rot.0.0) + (p.y * rot.0.1),
            y: (p.x * rot.1.0) + (p.y * rot.1.1),
        }
    }

    pub fn get_opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }

    pub fn get_disallowed(&self) -> Direction {
        self.get_opposite()
    }

    pub fn direction_get_unit_vector(&self) -> Coord {
        match self {
            Direction::Up => Coord::new(0, -1),
            Direction::Right => Coord::new(1, 0),
            Direction::Down => Coord::new(0, 1),
            Direction::Left => Coord::new(-1, 0),
        }
    }
}
