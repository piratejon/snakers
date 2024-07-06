use crate::pair::Pair;
use crate::direction::Direction;

pub type Coord = Pair<i32>;

impl Coord {
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x: x, y: y }
    }

    pub fn as_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn calculate_neighbor(&self,
                              direction: Direction)
        -> Coord
    {
        let uv = direction.direction_get_unit_vector();

        Coord {
            x: self.x + uv.x,
            y: self.y + uv.y,
        }
    }
}
