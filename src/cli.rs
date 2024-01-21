use string_builder::Builder;

use snake::*;

use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() {
  snake_game(get_input, draw);
}

fn get_input() -> InputType {
  enable_raw_mode().unwrap();
  let input = match read().unwrap() {
    Event::Key(KeyEvent {
      code: KeyCode::Char('q'), ..
    }) => InputType::Quit,
    Event::Key(KeyEvent { code: KeyCode::Up, ..  }) => InputType::Up,
    Event::Key(KeyEvent { code: KeyCode::Right, ..  }) => InputType::Right,
    Event::Key(KeyEvent { code: KeyCode::Down, ..  }) => InputType::Down,
    Event::Key(KeyEvent { code: KeyCode::Left, ..  }) => InputType::Left,
    Event::Key(_) => {
      // println!("{:?}", event);
      InputType::Nothing
    },
    _ => InputType::Nothing,
  };
  disable_raw_mode().unwrap();
  input
}

fn draw(world : &WorldType) {

  println!("+{0}+", "-".repeat(WORLD_DIMENSIONS.x as usize));

  for y in 0..WORLD_DIMENSIONS.y {

    let mut builder = Builder::new((WORLD_DIMENSIONS.x + 2) as usize);

    builder.append("|");

    for x in 0..WORLD_DIMENSIONS.x {
      match &world[y as usize][x as usize] {
        Item::Nothing  => builder.append(" "),
        Item::Food     => builder.append("O"),
        Item::SnakeBit => builder.append("S"),
        Item::SnakeHead => builder.append("%"),
        Item::SnakeTail => builder.append("*"),
      }
    }

    builder.append("|");

    println!("{0}", builder.string().unwrap());
  }

  println!("+{0}+", "-".repeat(WORLD_DIMENSIONS.x as usize));
}

