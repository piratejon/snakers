extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect,Point};

use snake::GameState;
use snake::GridType;
use snake::InputType;
use snake::Item;
use snake::StateTransition;

use std::time::Duration;

const WIDTH_PIXELS : u32 = 1200;
const HEIGHT_PIXELS : u32 = 800;

const GAME_TO_SCREEN_FACTOR : u32 = 50;

const WIDTH : u32 = WIDTH_PIXELS / GAME_TO_SCREEN_FACTOR;
const HEIGHT : u32 = HEIGHT_PIXELS / GAME_TO_SCREEN_FACTOR;

const FPS: f64 = 30.0;
const FPS_RATE: Duration = Duration::from_nanos(((1e9 as f64) / FPS) as u64);

const FRAMES_PER_TICK: u32 = 20;

struct SDLContext<'a> {
  color_index: u8,
  event_pump: &'a mut sdl2::EventPump,
  canvas : &'a mut sdl2::render::Canvas<sdl2::video::Window>,
  timer: &'a mut sdl2::TimerSubsystem,

  timer_freq: u64,
  start_frame_time: u64,
  last_frame_time: u64,
  frame_counter: u64,

  // frame_duration_ewma: u64,
}

trait SnakeGameRenderTrait {
  fn draw_food(&mut self, x: u32, y: u32);
  fn draw_snake(&mut self, x: u32, y: u32);
}

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();

  let window = video_subsystem.window("Snake.rs - SDL2 Driver", WIDTH_PIXELS, HEIGHT_PIXELS)
    .position(0,0)
    .build()
    .unwrap();

  let mut canvas = Some(window.into_canvas().build().unwrap());

  if let Some(ref mut canvas_here) = canvas {
    canvas_here.set_draw_color(Color::RGB(0,255,255));
    canvas_here.clear();
    canvas_here.present();
  }

  let timer = sdl_context.timer();

  let mut ctx: SDLContext = SDLContext {
    color_index: 0,
    canvas: &mut canvas.unwrap(),
    event_pump: &mut sdl_context.event_pump().unwrap(),
    timer: &mut timer.unwrap(),
    timer_freq: 0,
    start_frame_time: 0,
    last_frame_time: 0,
    frame_counter: 0,
  };

  ctx.last_frame_time = sdl2::TimerSubsystem::performance_counter(&ctx.timer);
  ctx.start_frame_time = ctx.last_frame_time;
  ctx.timer_freq = sdl2::TimerSubsystem::performance_frequency(&ctx.timer);


  let mut game = snake::GameState::new(WIDTH, HEIGHT);

  let mut frame_counter = 1;

  loop {
    ctx.draw(game.get_world());
    let input = ctx.get_input();
    match game.handle_input(input) {
      StateTransition::Stop => break,
      _ => (),
    }

    if (frame_counter % FRAMES_PER_TICK) == 0 {
      match game.update_state() {
        StateTransition::Stop => break,
        _ => (),
      }
    }

    frame_counter += 1;
  }
}

impl SnakeGameRenderTrait for SDLContext<'_> {

  fn draw_food(&mut self, x: u32, y: u32) {
    self.canvas.set_draw_color(Color::RGB(200, 200, 20));
    let _ = self.canvas.fill_rect(Rect::new(
      ((x * GAME_TO_SCREEN_FACTOR) + 2) as i32,
      ((y * GAME_TO_SCREEN_FACTOR) + 2) as i32,
      GAME_TO_SCREEN_FACTOR - 4,
      GAME_TO_SCREEN_FACTOR - 4,
    ));
  }

  fn draw_snake(&mut self, x: u32, y: u32) {
    self.canvas.set_draw_color(Color::RGB(0, 200, 50));
    let _ = self.canvas.fill_rect(Rect::new(
      ((x * GAME_TO_SCREEN_FACTOR) + 2) as i32,
      ((y * GAME_TO_SCREEN_FACTOR) + 2) as i32,
      GAME_TO_SCREEN_FACTOR - 4,
      GAME_TO_SCREEN_FACTOR - 4,
    ));
  }

}

impl SDLContext<'_> {

  fn draw(&mut self, grid: &snake::GridType) {

    self.frame_counter += 1;

    // update background
    self.color_index = (self.color_index + 1) % 255;
    self.canvas.set_draw_color(Color::RGB(self.color_index, 64, 255 - self.color_index));
    self.canvas.clear();
    // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
      
      /*
      game logic down here
      */

    for y in 0..HEIGHT {
      for x in 0..WIDTH {
        match &grid[y as usize][x as usize] {
          Item::Nothing => (),
          Item::Food => self.draw_food(x, y),
          Item::SnakeBit | Item::SnakeHead | Item::SnakeTail => self.draw_snake(x, y),
        }
      }
    }

    let cur_time : u64 = sdl2::TimerSubsystem::performance_counter(&self.timer);
    // println!("delta: {}", cur_time - self.last_frame_time);

    let frame_elapsed : u64 = cur_time - self.last_frame_time;

    let time_to_next_frame = FPS_RATE - Duration::from_secs(frame_elapsed / self.timer_freq);

    if (self.frame_counter % 300) == 0 {
      println!("frames: {}; FPS: {}; cur_time: {}",
        self.frame_counter,
        1e9 * ((self.frame_counter as f64) / ((cur_time - self.start_frame_time) as f64)),
        cur_time);
    }

    self.canvas.present();

    if time_to_next_frame > Duration::from_secs(0) {
      std::thread::sleep(time_to_next_frame);
    }

    self.last_frame_time = cur_time;
  }

  fn get_input(&mut self) -> snake::InputType {
    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } |
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
          return snake::InputType::Quit;
        },
        Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
          return snake::InputType::Quit;
        },
        Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
          return snake::InputType::Up;
        },
        Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
          return snake::InputType::Right;
        },
        Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
          return snake::InputType::Down;
        },
        Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
          return snake::InputType::Left;
        },
        _ => snake::InputType::Nothing,
      };
    }
    return snake::InputType::Nothing;
  }
}
