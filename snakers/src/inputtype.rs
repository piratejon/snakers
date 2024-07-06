use crate::direction::Direction;

#[derive(PartialEq, Debug,Copy,Clone)]
pub enum InputType {
    Nothing,
    Up,
    Right,
    Down,
    Left,
    Quit,
}

impl InputType {
    pub fn get_direction(&self) -> Option<Direction> {
        match self {
            Self::Up => Some(Direction::Up),
            Self::Right => Some(Direction::Right),
            Self::Down => Some(Direction::Down),
            Self::Left => Some(Direction::Left),
            _ => None,
        }
    }
}
