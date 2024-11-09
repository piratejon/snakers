use std::ops::Sub;

use crate::direction::Direction;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pair<T: Copy> {
    pub x: T,
    pub y: T,
}

impl<T> Sub<&Pair<T>> for &Pair<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Pair<T>;

    fn sub(self, rhs: &Pair<T>) -> Pair<T> {
        return Pair::<T> {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

impl<T> Pair<T>
where
    T: Copy + std::cmp::PartialOrd + std::ops::Sub<Output = T>,
{
    pub fn get_x(&self) -> T {
        self.x
    }

    pub fn get_y(&self) -> T {
        self.y
    }

    pub fn unit_vector_to(&self, other: &Self) -> Pair<T> {
        self - other
    }

    pub fn direction_to(&self, other: &Self) -> Option<Direction> {
        if self.y == other.y {
            if self.x < other.x {
                return Some(Direction::Right);
            } else if self.x > other.x {
                return Some(Direction::Left);
            }
        } else if self.x == other.x {
            if self.y < other.y {
                return Some(Direction::Down);
            } else if self.y > other.y {
                return Some(Direction::Up);
            }
        }

        return None;
    }
}

impl<T> std::fmt::Display for Pair<T>
where
    T: std::fmt::Display + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:2},{:2})", self.x, self.y)
    }
}
