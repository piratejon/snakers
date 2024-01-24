extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use snake::GridType;
use snake::InputType;
use snake::snake_game;
use snake::ContextTrait;

const WIDTH_PIXELS : u32 = 1000;
const HEIGHT_PIXELS : u32 = 800;

const WIDTH : u32 = WIDTH_PIXELS / 20;
const HEIGHT : u32 = HEIGHT_PIXELS / 20;

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

  let mut ctx: SDLContext = SDLContext {
    color_index: 0,
    canvas: &mut canvas.unwrap(),
    event_pump: &mut sdl_context.event_pump().unwrap(),
  };

  snake_game(WIDTH, HEIGHT, &mut ctx);
}

struct SDLContext<'a> {
  color_index: u8,
  event_pump: &'a mut sdl2::EventPump,
  canvas : &'a mut sdl2::render::Canvas<sdl2::video::Window>,
}

impl ContextTrait for SDLContext<'_> {

  fn draw(&mut self, grid: &snake::GridType) {
    self.color_index = (self.color_index + 1) % 255;
    self.canvas.set_draw_color(Color::RGB(self.color_index, 64, 255 - self.color_index));
    self.canvas.clear();
    self.canvas.present();
    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
      
      /*
      game logic down here
      */
  }

  fn get_input(&mut self) -> snake::InputType {
    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } |
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
          return snake::InputType::Quit;
        },
        _ => snake::InputType::Nothing,
      };
    }
    return snake::InputType::Nothing;
  }
}
