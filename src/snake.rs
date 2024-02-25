use std::collections::LinkedList;

use std::ops::Sub;

use rand::Rng;

#[derive(Copy, Clone, Debug)]
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

pub type Coord = Pair<i32>;

impl Coord {
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x: x, y: y }
    }

    pub fn as_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

const INITIAL_SNAKE_LENGTH: i32 = 6;
const SNAKE_GROWTH_PER_FOOD: i32 = 3;

#[derive(Debug, PartialEq)]
pub enum ItemType {
    Nothing,
    SnakeHead,
    SnakeBit,
    SnakeTail,
    Food,
}

#[derive(Copy,Clone,Debug)]
pub struct CoordWithDirection {
    pub dir_next: Direction,
    pub coord: Coord,
    pub dir_prev: Option<Direction>,
}

impl CoordWithDirection {
    pub fn direction_to(&self, other: &Self) -> Option<Direction> {
        self.coord.direction_to(&other.coord)
    }
}

pub struct SnakeType {
    direction: Direction,
    body: LinkedList<CoordWithDirection>,
    growing: i32,
}

impl SnakeType {
    pub fn get_direction(&self) -> Direction {
        self.direction
    }
    pub fn get_body(&self) -> &LinkedList<CoordWithDirection> {
        &(self.body)
    }
    pub fn get_growing(&self) -> i32 {
        self.growing
    }
}

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

    fn get_disallowed(&self) -> Direction {
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

pub type GridType = Vec<Vec<ItemType>>;

pub struct GameState {
    // grid size
    width: u32,
    height: u32,

    world: GridType,
    snake: SnakeType,

    // logical game state bounds
    xrange: (i32, i32),
    yrange: (i32, i32),

    pending_input: InputType,
}

fn init_grid(width: u32, height: u32) -> Vec<Vec<ItemType>> {
    let mut row_vec: Vec<Vec<ItemType>> = Vec::with_capacity(height as usize);
    for _ in 0..row_vec.capacity() {
        let mut row = Vec::with_capacity(width as usize);
        for _ in 0..width {
            row.push(ItemType::Nothing);
        }
        row_vec.push(row);
    }
    return row_vec;
}

pub enum StateTransition {
    Continue,
    Stop,
}

fn make_coordinate_range(size: u32) -> (i32, i32) {
    match size % 2 {
        0 => {
            let half = (size / 2) as i32;
            return (-half, half - 1);
        }
        _ => {
            let half = ((size - 1) / 2) as i32;
            return (-half, half);
        }
    }
}

impl GameState {
    pub fn new(width: u32, height: u32) -> Self {
        let mut state = GameState {
            width: width,
            height: height,

            world: init_grid(width, height),

            snake: SnakeType {
                direction: Direction::Up,
                body: LinkedList::new(),
                growing: 0,
            },

            xrange: make_coordinate_range(width),
            yrange: make_coordinate_range(height),

            pending_input: InputType::Nothing,
        };

        state.initialize_snake();

        state.drop_new_food();

        state
    }

    pub fn get_world(&self) -> &GridType {
        &self.world
    }

    pub fn get_snake(&self) -> &SnakeType {
        &self.snake
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn handle_input(&mut self, input: InputType) -> StateTransition {

        match input {
            InputType::Quit => StateTransition::Stop,
            InputType::Nothing => StateTransition::Continue,
            _ => {
                self.pending_input = input;
                // println!("handling {:?}", k);
                return StateTransition::Continue;
            },
        }
    }

    pub fn update_state(&mut self) -> StateTransition {
        // apply the held keypress
        match self.process_input() {
            StateTransition::Continue => self.try_move_snake(),
            s => s,
        }
    }

    fn process_input(&mut self) -> StateTransition {
        match self.pending_input {
            d @ InputType::Up
            | d @ InputType::Right
            | d @ InputType::Down
            | d @ InputType::Left => {
                self.pending_input = InputType::Nothing;
                return self.handle_direction(input_get_direction(d).unwrap())
            },
            InputType::Quit => StateTransition::Stop,
            _ => StateTransition::Continue,
        }
    }

    fn snake_can_move(&self, target: &CoordWithDirection) -> bool {
        match self[&target.coord] {
            ItemType::Nothing => true,
            ItemType::Food => true,
            // if size is not increasing, then the current tail square will be open
            ItemType::SnakeTail => self.snake.growing <= 0,
            _ => {
                println!("{:#?} has {:#?}", target, self[&target.coord]);
                false
            }
        }
    }

    fn advance_head(&mut self, new_head: &CoordWithDirection) {
        // advance the head
        let mut old_head = self.snake.body.front_mut().unwrap();
        old_head.dir_next = new_head.dir_prev.expect("head has elements behind it").get_opposite();
        let coord = old_head.coord.clone();
        self[&coord] = ItemType::SnakeBit;

        self.snake.body.push_front(*new_head);
        self[&new_head.coord] = ItemType::SnakeHead;
    }

    fn bring_up_tail(&mut self) {
        let old_tail = self.snake.body.pop_back().unwrap().clone();
        self[&old_tail.coord] = ItemType::Nothing;

        let new_tail: &mut CoordWithDirection = self.snake.body.back_mut().unwrap();
        let coord = new_tail.coord.clone();
        new_tail.dir_prev = None;
        self[&coord] = ItemType::SnakeTail;
    }

    fn move_snake(&mut self, new_head: &CoordWithDirection) {
        if self[&new_head.coord] == ItemType::Food {
            self.snake.growing += SNAKE_GROWTH_PER_FOOD;
            self.drop_new_food();
        }

        self.advance_head(new_head);

        if self.snake.growing <= 0 {
            // bring up the tail by one
            self.bring_up_tail();

            if self.snake.growing < 0 {
                // bring up the tail by one more
                self.bring_up_tail();

                self.snake.growing += 1;
            }
        } else if self.snake.growing > 0 {
            /*
             * When we are growing, the tail does not need to move up. the head already moved up so
             * we are done.
             * */
            self.snake.growing -= 1;
        }
    }

    fn drop_new_food(&mut self) {
        for _ in 0..100 {
            let at = (
                rand::thread_rng().gen_range(self.xrange.0..=self.xrange.1),
                rand::thread_rng().gen_range(self.yrange.0..=self.yrange.1),
            );
            if self[&at] == ItemType::Nothing {
                self[&at] = ItemType::Food;
                break;
            }
        }
    }

    fn initialize_snake(&mut self) {
        for y in 0..INITIAL_SNAKE_LENGTH {
            let at = Coord { x: 0, y: y };

            println!("init snake: x: {}, y: {}", at.x, at.y);

            let dir_next: Direction = Direction::Up;
            let mut dir_prev: Option<Direction> = Some(Direction::Down);

            if y == 0 {
                self[&at] = ItemType::SnakeHead;
                self.snake.direction = Direction::Up;
            } else if y < (INITIAL_SNAKE_LENGTH - 1) {
                self[&at] = ItemType::SnakeBit;
            } else {
                self[&at] = ItemType::SnakeTail;
                dir_prev = None;
            }

            self.snake.body.push_back(CoordWithDirection {
                dir_next: dir_next,
                coord: at,
                dir_prev: dir_prev,
            });
        }
    }

    fn print_snake(&self) {
        for s in self.snake.body.iter() {
            println!("{:?}:{}:{:?}", s.dir_next, s.coord, s.dir_prev);
        }
    }

    fn try_move_snake(&mut self) -> StateTransition {

        // self.print_snake();

        let old_head = self.snake.body.front().unwrap();

        match self.try_create_target(&old_head, &self.snake.direction) {
            Some(new_head) => {
                if self.snake_can_move(&new_head) {
                    // println!("old_head:{}; new_head:{}, dir:{:#?}", old_head, new_head, s.snake.direction);
                    self.move_snake(&new_head);
                    return StateTransition::Continue;
                } else {
                    return StateTransition::Stop;
                }
            }
            None => StateTransition::Stop,
        }
    }

    fn try_create_target(&self, a: &CoordWithDirection, d: &Direction) -> Option<CoordWithDirection> {

        let uv = d.direction_get_unit_vector();

        let new_x = a.coord.x + uv.x;
        let new_y = a.coord.y + uv.y;

        let target = Coord { x: new_x, y: new_y };

        if new_x >= self.xrange.0 {
            if new_x <= self.xrange.1 {
                if new_y >= self.yrange.0 {
                    if new_y <= self.yrange.1 {
                        let out = CoordWithDirection {
                            dir_next: *d,
                            coord: target,
                            dir_prev: Some(d.get_opposite()),
                        };
                        // println!("created target {:?} from {:?}+{:?}", target, a, d);
                        return Some(out);
                    }
                }
            }
        }

        println!("failed to create target from {:?} and {:?}: {}", a, d, target);

        None
    }

    pub fn game_to_grid(&self, at: &Coord) -> Coord {
        Coord {
            x: at.x - self.xrange.0,
            y: at.y - self.yrange.0,
        }
    }

    pub fn game_to_grid_tuple(&self, at: &(i32, i32)) -> (usize, usize) {
        let g = (
            (at.0 - self.xrange.0) as usize,
            (at.1 - self.yrange.0) as usize,
        );
        // println!("({},{}) -> ({},{})", at.0, at.1, g.0, g.1);
        return (g.0, g.1);
    }

    fn handle_direction(&mut self, direction: Direction) -> StateTransition {
        if self.snake.direction != direction.get_disallowed() {
            println!(
                "changing direction from {:?} to {:?}",
                self.snake.direction, direction
            );
            self.snake.direction = direction;
        } else {
            println!(
                "not changing direction from {:?} to {:?}",
                self.snake.direction, direction
            );
        }
        StateTransition::Continue
    }
}

impl std::ops::Index<&(i32, i32)> for GameState {
    type Output = ItemType;

    fn index(&self, at: &(i32, i32)) -> &Self::Output {
        let g = self.game_to_grid_tuple(&at);
        return &self.world[g.1][g.0];
    }
}

impl std::ops::IndexMut<&(i32, i32)> for GameState {
    fn index_mut(&mut self, at: &(i32, i32)) -> &mut Self::Output {
        let g = self.game_to_grid_tuple(&at);
        &mut self.world[g.1][g.0]
    }
}

impl std::ops::Index<&Coord> for GameState {
    type Output = ItemType;

    fn index(&self, at: &Coord) -> &Self::Output {
        &self[&(at.x, at.y)]
    }
}

impl std::ops::IndexMut<&Coord> for GameState {
    fn index_mut(&mut self, at: &Coord) -> &mut Self::Output {
        &mut self[&(at.x, at.y)]
    }
}

fn input_get_direction(input: InputType) -> Option<Direction> {
    match input {
        InputType::Up => Some(Direction::Up),
        InputType::Right => Some(Direction::Right),
        InputType::Down => Some(Direction::Down),
        InputType::Left => Some(Direction::Left),
        _ => None,
    }
}

// TODO use direction
#[derive(PartialEq, Debug,Copy,Clone)]
pub enum InputType {
    Nothing,
    Up,
    Right,
    Down,
    Left,
    Quit,
}
