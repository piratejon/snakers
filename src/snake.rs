
use std::collections::LinkedList;
use init_array::init_boxed_array;

use rand::Rng;

trait CoordinatePairTrait {}

impl CoordinatePairTrait for i32 {}
impl CoordinatePairTrait for u32 {}

#[derive(Copy,Clone,Debug)]
pub struct Pair<T: CoordinatePairTrait> {
  pub x : T,
  pub y : T,
}

impl<T: CoordinatePairTrait + std::fmt::Display> std::fmt::Display for Pair<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "({:2},{:2})", self.x, self.y)
  }
}

type UnitVector = Pair<i32>;
type Coord = Pair<u32>;

pub const WORLD_DIMENSIONS : Coord = Coord {
  x: 48,
  y: 18,
};

const INITIAL_SNAKE_LENGTH : u32 = 6;

const SNAKE_GROWTH_PER_FOOD : i32 = 3;

#[derive(Debug,PartialEq)]
pub enum Item {
  Nothing,
  SnakeHead,
  SnakeBit,
  SnakeTail,
  Food,
}

pub type WorldType = Box<[
  Box<[Item; WORLD_DIMENSIONS.x as usize]>;
  WORLD_DIMENSIONS.y as usize
]>;

struct Snake {
  direction : Direction,
  body : LinkedList<Coord>,
  growing : i32,
}

// specific values so we can use as array indices
#[derive(PartialEq,Copy,Clone,Debug)]
enum Direction {
  Up,
  Right,
  Down,
  Left,
}

struct GameState<'a> {
  world: &'a mut WorldType,
  snake: &'a mut Snake,
}

enum StateTransition {
  Continue,
  Stop,
}

pub fn snake_game(
  get_input : fn()           -> InputType,
  draw      : fn(&WorldType) -> (),
) {

  let mut state = GameState {
    world: &mut init_boxed_array(|_| {
      init_boxed_array(|_| {
        Item::Nothing
      })
    }),
    snake: &mut Snake {
      direction: Direction::Up,
      body: LinkedList::new(),
      growing : 0,
    },
  };


  state.world[3][3] = Item::Food;

  initialize_snake(&mut state);

  loop {

    draw(&state.world);

    let input = get_input();

    match handle_input(input, &mut state) {
      StateTransition::Stop => break,
      _ => (),
    }

    match update_state(&mut state) {
      StateTransition::Stop => break,
      _ => (),
    }
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

fn direction_get_disallowed(direction: &Direction) -> Direction {
  match direction {
    Direction::Up => Direction::Down,
    Direction::Right => Direction::Left,
    Direction::Down => Direction::Up,
    Direction::Left => Direction::Right,
  }
}

fn direction_get_unit_vector(direction: Direction) -> UnitVector {
  match direction {
    Direction::Up => UnitVector { x: 0, y: -1 },
    Direction::Right => UnitVector { x: 1, y: 0 },
    Direction::Down => UnitVector { x: 0, y: 1 },
    Direction::Left => UnitVector { x: -1, y: 0 },
  }
}

fn update_state(state: &mut GameState) -> StateTransition {
  try_move_snake(state)
}

fn snake_can_move(world: &WorldType, target: &Coord) -> bool {
  match world[target.y as usize][target.x as usize] {
    Item::Nothing => true,
    Item::Food => true,
    _ => {
      println!("{} has {:#?}", target, world[target.y as usize][target.x as usize]);
      false
    },
  }
}

fn try_create_target(a: &Coord, d: &UnitVector) -> Option<Coord> {
  let new_x = a.x.checked_add_signed(d.x).unwrap();
  if new_x >= 0 && new_x < WORLD_DIMENSIONS.x {
    let new_y = a.y.checked_add_signed(d.y).unwrap();
    if new_y >= 0 && new_y  < WORLD_DIMENSIONS.y {
      let target = Coord {x: new_x, y: new_y};
      println!("created target {} from {}+{}",
        target, a, d);
      return Some(target);
    } else {
      println!("failed to create target from {} and {}",
        a, d
      );
    }
  }

  None
}

fn try_move_snake(s: &mut GameState) -> StateTransition {
  let old_head = s.snake.body.front().unwrap();

  match try_create_target(old_head, &direction_get_unit_vector(s.snake.direction)) {
    Some(new_head) =>
      if snake_can_move(s.world, &new_head) {
        println!("old_head:{}; new_head:{}, dir:{:#?}",
          old_head, new_head, s.snake.direction);
        move_snake(s, &new_head);
        return StateTransition::Continue;
      } else {
        return StateTransition::Stop;
      }
    None => StateTransition::Stop,
  }
}

fn drop_new_food(s: &mut GameState) {
  for attempt in 0..10 {
    let rnd_x = rand::thread_rng().gen_range(0..WORLD_DIMENSIONS.x) as usize;
    let rnd_y = rand::thread_rng().gen_range(0..WORLD_DIMENSIONS.y) as usize;
    if s.world[rnd_y][rnd_x] == Item::Nothing {
      s.world[rnd_y][rnd_x] = Item::Food;
      break
    }
  }
}

fn move_snake(s: &mut GameState, new_head: &Coord) {

  if s.world[new_head.y as usize][new_head.x as usize] == Item::Food {
    s.snake.growing += SNAKE_GROWTH_PER_FOOD;
    drop_new_food(s);
  }

  // advance the head
  let old_head = s.snake.body.front().unwrap();
  s.world[old_head.y as usize][old_head.x as usize] = Item::SnakeBit;

  s.snake.body.push_front(*new_head);
  s.world[new_head.y as usize][new_head.x as usize] = Item::SnakeHead;

  if s.snake.growing <= 0 {
    // bring up the tail by one
    let old_tail = s.snake.body.pop_back().unwrap();
    s.world[old_tail.y as usize][old_tail.x as usize] = Item::Nothing;

    let new_tail = s.snake.body.back().unwrap();
    s.world[new_tail.y as usize][new_tail.x as usize] = Item::SnakeTail;

    if s.snake.growing < 0 {
      // bring up the tail by one more
      let old_tail = s.snake.body.pop_back().unwrap();
      s.world[old_tail.y as usize][old_tail.x as usize] = Item::Nothing;

      let new_tail = s.snake.body.back().unwrap();
      s.world[new_tail.y as usize][new_tail.x as usize] = Item::SnakeTail;

      s.snake.growing += 1;
    }
  } else if s.snake.growing > 0 {
    // just advance the head, do not bring up the tail
    s.snake.growing -= 1;
  }

  println!("new_tail:{}", s.snake.body.back().unwrap());
}

fn initialize_snake(state: &mut GameState) {
  let x = (WORLD_DIMENSIONS.x / 2) as usize;
  let y = (WORLD_DIMENSIONS.y - INITIAL_SNAKE_LENGTH) / 2;

  state.world[y as usize][x] = Item::SnakeHead;
  state.snake.body.push_back(Coord{x:x as u32,y:y});

  for y in y+1..(y+(INITIAL_SNAKE_LENGTH as u32)-1) {
    state.world[y as usize][x] = Item::SnakeBit;
    state.snake.body.push_back(Coord{x:x as u32,y:y});
  }
  state.world[(y+INITIAL_SNAKE_LENGTH-1) as usize][x] = Item::SnakeTail;
  state.snake.body.push_back(Coord{x:x as u32,y:(y+INITIAL_SNAKE_LENGTH-1)});

  state.snake.direction = Direction::Up;
}

#[derive(PartialEq)]
pub enum InputType {
  Nothing,
  Up,
  Right,
  Down,
  Left,
  Quit,
}

fn handle_input(
  input : InputType,
  s: &mut GameState,
) -> StateTransition {
  match input {
    d @ InputType::Up |
    d @ InputType::Right |
    d @ InputType::Down |
    d @ InputType::Left =>
      handle_direction(s, input_get_direction(d).unwrap()),
    InputType::Quit => StateTransition::Stop,
    _ => StateTransition::Continue,
  }
}

fn handle_direction(s: &mut GameState, direction: Direction) -> StateTransition {
  if s.snake.direction != direction_get_disallowed(&direction) {
    println!("changing direction from {:?} to {:?}",
      s.snake.direction, direction);
    s.snake.direction = direction;
  } else {
    println!("not changing direction from {:?} to {:?}",
      s.snake.direction, direction);
  }
  StateTransition::Continue
}

