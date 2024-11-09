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

const UP: Coord = Coord { x: 0, y: -1 };
const RIGHT: Coord = Coord { x: 1, y: 0 };
const LEFT: Coord = Coord { x: -1, y: 0 };
const DOWN: Coord = Coord { x: 0, y: 1 };

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
            Direction::Up => UP,
            Direction::Right => RIGHT,
            Direction::Down => DOWN,
            Direction::Left => LEFT,
        }
    }

    pub fn direction_from_unit_vector(p: &Coord) -> Direction {
        match p {
            &UP => Direction::Up,
            &RIGHT => Direction::Right,
            &LEFT => Direction::Left,
            &DOWN => Direction::Down,
            _ => panic!(),
        }
    }
}

