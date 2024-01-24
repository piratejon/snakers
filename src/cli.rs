use string_builder::Builder;
use std::time::Duration;

use snake::*;

use crossterm::event::{read, poll, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

const WIDTH : u32 = 48;
const HEIGHT : u32 = 18;

fn main() {
  let mut ctx: CliContext = CliContext {};
  snake_game(WIDTH, HEIGHT, &mut ctx);
}

struct CliContext {}

impl snake::ContextTrait for CliContext {

  fn get_input(&self) -> InputType {
    let mut input = InputType::Nothing;
    enable_raw_mode().unwrap();
    // this pol does not work
    let poll_result = poll(Duration::from_millis(750));
    if let Ok(_) = poll_result {
      input = match read().unwrap() {
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
    }
    disable_raw_mode().unwrap();
    input
  }

  fn draw(&self, world : &GridType) {

    println!("+{0}+", "-".repeat(WIDTH as usize));

    for y in 0..HEIGHT {

      let mut builder = Builder::new((WIDTH + 2) as usize);

      builder.append("|");

      for x in 0..WIDTH {
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

    println!("+{0}+", "-".repeat(WIDTH as usize));
  }

}
