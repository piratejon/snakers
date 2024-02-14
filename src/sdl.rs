use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
// use sdl2::render::Canvas;
// use sdl2::video::Window;
// use sdl2::EventPump;

use snake::Direction;
use snake::ItemType;
use snake::StateTransition;

use std::time::Duration;

const WIDTH_PIXELS: u32 = 1200;
const HEIGHT_PIXELS: u32 = 800;

const GAME_TO_SCREEN_FACTOR: u32 = 50;

const CELL_MARGIN: u32 = 4;

const WIDTH: u32 = WIDTH_PIXELS / GAME_TO_SCREEN_FACTOR;
const HEIGHT: u32 = HEIGHT_PIXELS / GAME_TO_SCREEN_FACTOR;

const FRAMES_PER_SECOND: f64 = 30.0;
const FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / FRAMES_PER_SECOND) as u64);

const RATE_LIMITED: bool = true;

const TICKS_PER_SECOND: f64 = 1.5;
const TICK_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / TICKS_PER_SECOND) as u64);

const FOOD_COLOR: Color = Color::RGB(200, 200, 20);
const SNAKE_COLOR: Color = Color::RGB(0, 200, 50);
const SNAKE_COLOR_DARK: Color = Color::RGB(0, 150, 60);

struct SDLContext<'a> {
    color_index: u8,
    event_pump: &'a mut sdl2::EventPump,
    canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>,
    timer: &'a mut sdl2::TimerSubsystem,

    timer_freq: u64,
    start_time: u64,
    last_frame_time: u64,
    last_tick_time: u64,
    frame_counter: u64,
    tick_counter: u64,
    // frame_duration_ewma: u64,
}

trait SnakeGameRenderTrait {
    fn draw_food(&mut self, at: &(usize, usize));
    fn draw_snake(&mut self, at: &(usize, usize));
    fn draw_animated_snake(&mut self, game: &snake::GameState);
    fn connect_snake_bits(
        &mut self,
        game: &snake::GameState,
        prev: &snake::Coord,
        next: &snake::Coord,
    );
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Snake.rs - SDL2 Driver", WIDTH_PIXELS, HEIGHT_PIXELS)
        .position(0, 0)
        .build()
        .unwrap();

    let mut canvas = Some(window.into_canvas().build().unwrap());

    if let Some(ref mut canvas_here) = canvas {
        canvas_here.set_draw_color(Color::RGB(0, 255, 255));
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
        start_time: 0,
        last_frame_time: 0,
        last_tick_time: 0,
        frame_counter: 0,
        tick_counter: 0,
    };

    ctx.last_frame_time = sdl2::TimerSubsystem::performance_counter(&ctx.timer);
    ctx.start_time = ctx.last_frame_time;
    ctx.timer_freq = sdl2::TimerSubsystem::performance_frequency(&ctx.timer);

    let mut game = snake::GameState::new(WIDTH, HEIGHT);

    ctx.last_tick_time = sdl2::TimerSubsystem::performance_counter(&ctx.timer);

    let mut last_tick_frame_number = ctx.frame_counter;

    loop {
        ctx.draw(&game);
        let input = ctx.get_input();
        match game.handle_input(input) {
            StateTransition::Stop => break,
            _ => (),
        }

        let cur_time = sdl2::TimerSubsystem::performance_counter(&ctx.timer);

        if Duration::from_nanos(cur_time - ctx.last_tick_time) >= TICK_DURATION {
            println!(
                "frames: {}; Tick FPS: {:.02}; Avg FPS: {:.02}",
                ctx.frame_counter,
                1e9 * (((ctx.frame_counter - last_tick_frame_number) as f64)
                    / ((cur_time - ctx.last_tick_time) as f64)),
                1e9 * ((ctx.frame_counter as f64) / ((cur_time - ctx.start_time) as f64)),
            );

            match game.update_state() {
                StateTransition::Stop => break,
                _ => (),
            }

            last_tick_frame_number = ctx.frame_counter;
            ctx.tick_counter += 1;
            ctx.last_tick_time = cur_time;
        }

        ctx.frame_counter += 1;
    }
}

impl SnakeGameRenderTrait for SDLContext<'_> {
    fn draw_food(&mut self, at: &(usize, usize)) {
        self.canvas.set_draw_color(FOOD_COLOR);
        let _ = self.canvas.fill_rect(Rect::new(
            ((at.0 as u32 * GAME_TO_SCREEN_FACTOR) + CELL_MARGIN) as i32,
            ((at.1 as u32 * GAME_TO_SCREEN_FACTOR) + CELL_MARGIN) as i32,
            GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2),
            GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2),
        ));
    }

    fn draw_snake(&mut self, at: &(usize, usize)) {
        self.canvas.set_draw_color(SNAKE_COLOR);
        let _ = self.canvas.fill_rect(Rect::new(
            ((at.0 as u32 * GAME_TO_SCREEN_FACTOR) + CELL_MARGIN) as i32,
            ((at.1 as u32 * GAME_TO_SCREEN_FACTOR) + CELL_MARGIN) as i32,
            GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2),
            GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2),
        ));
    }

    /*
     * draw the snake by iterating the linked list
     * the first one is drawn first, then the rest
     * are drawn connecting to the previous
     * */
    fn draw_animated_snake(&mut self, game: &snake::GameState) {
        self.canvas.set_draw_color(SNAKE_COLOR);
        // let mut iter = game.get_snake().body.iter();
        let mut iter = game.get_snake().get_body().iter();

        if let Some(first) = iter.next() {
            let mut prev = first;

            self.draw_snake(&game.game_to_grid(&prev.as_tuple()));

            for next in iter {
                self.draw_snake(&game.game_to_grid(&next.as_tuple()));
                self.connect_snake_bits(game, prev, next);
                prev = next;
            }
        }
    }

    fn connect_snake_bits(
        &mut self,
        game: &snake::GameState,
        prev: &snake::Coord,
        next: &snake::Coord,
    ) {
        self.canvas.set_draw_color(SNAKE_COLOR_DARK);

        let p = game.game_to_grid(&prev.as_tuple());
        let n = game.game_to_grid(&next.as_tuple());

        let GF: i32 = GAME_TO_SCREEN_FACTOR as i32;
        let CM: i32 = CELL_MARGIN as i32;
        let CM2: u32 = CM as u32 * 2;

        let cx: i32 = (p.0 as i32 * GF);
        let cy: i32 = (p.1 as i32 * GF);

        if let Some(dir) = next.direction_to(prev) {
            let xys = match dir {
                Direction::Up => (cx + CM, cy - CM + GF as i32, GF as u32 - CM2, CM2),
                Direction::Down => (cx + CM, cy - CM, GF as u32 - CM2, CM2),
                Direction::Left => (cx - CM + GF as i32, cy + CM, CM2, GF as u32 - CM2),
                Direction::Right => (cx - CM, cy + CM, CM2, GF as u32 - CM2),
                _ => {
                    println!("no dir: {} -> {}", prev, next);
                    (0, 0, 0, 0)
                }
            };

            let _ = self.canvas.fill_rect(Rect::new(xys.0, xys.1, xys.2, xys.3));
        }
    }
}

impl SDLContext<'_> {
    fn draw(&mut self, game: &snake::GameState) {
        // update background
        self.color_index = (self.color_index + 1) % 255;
        // self.canvas.set_draw_color(Color::RGB(self.color_index, 64, 255 - self.color_index));
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        self.canvas.clear();

        /*
        game logic down here
        */

        // render the grid
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                match &game.get_world()[y][x] {
                    ItemType::Nothing => (),
                    ItemType::Food => self.draw_food(&(x, y)),
                    // ItemType::SnakeBit | ItemType::SnakeHead | ItemType::SnakeTail => self.draw_snake(x, y),
                    _ => (),
                }
            }
        }

        self.draw_animated_snake(&game);

        self.canvas.present();

        if RATE_LIMITED {
            let cur_time: u64 = sdl2::TimerSubsystem::performance_counter(&self.timer);
            let frame_elapsed: u64 = cur_time - self.last_frame_time;
            let time_to_next_frame =
                FRAME_DURATION - Duration::from_secs(frame_elapsed / self.timer_freq);

            if time_to_next_frame > Duration::from_secs(0) {
                std::thread::sleep(time_to_next_frame);
            }

            self.last_frame_time = cur_time;
        }
    }

    fn get_input(&mut self) -> snake::InputType {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return snake::InputType::Quit;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    return snake::InputType::Quit;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    return snake::InputType::Up;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    return snake::InputType::Right;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    return snake::InputType::Down;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    return snake::InputType::Left;
                }
                _ => snake::InputType::Nothing,
            };
        }
        return snake::InputType::Nothing;
    }
}
