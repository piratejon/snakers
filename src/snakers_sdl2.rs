
// provides pie and filled_pie for sdl2::render::Canvas
use sdl2::gfx::primitives::DrawRenderer;

mod textures;

use crate::textures::SnakeTextureManager;

use ::snake::InputType;
use ::snake::ItemType;
use ::snake::GameState;
// use ::snake::CoordWithDirection;
use ::snake::Direction;
use ::snake::StateTransition;

const WIDTH_PIXELS: u32 = 1200;
const HEIGHT_PIXELS: u32 = 750;

const GAME_TO_PIXEL: u32 = 50;

const CELL_MARGIN_PX: u32 = 4;

const WIDTH: u32 = WIDTH_PIXELS / GAME_TO_PIXEL;
const HEIGHT: u32 = HEIGHT_PIXELS / GAME_TO_PIXEL;

const FRAMES_PER_SECOND: f64 = 30.0;
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos((1_000_000_000.0 / FRAMES_PER_SECOND) as u64);

const RATE_LIMITED: bool = true;

const TICKS_PER_SECOND: f64 = 0.5;
const TICK_DURATION: std::time::Duration = std::time::Duration::from_nanos((1_000_000_000.0 / TICKS_PER_SECOND) as u64);

const FOOD_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(200, 200, 20);
const RED: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 0, 0);
const BLUE: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 255);

/*
 * lifetime notes
 * event_pump and timer are only used in this file.
 * canvas may be needed by texturemanager to render things -- pass a reference on point of need
 * stm also needs canvas or texture_creator to create textures, during init.
 * */

struct SDLContext<'a> {
    event_pump: sdl2::EventPump,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    timer: sdl2::TimerSubsystem,

    timer_freq: u64,
    start_time: u64,
    last_frame_time: u64,
    last_tick_time: u64,
    frame_counter: u64,
    tick_counter: u64,
    // frame_duration_ewma: u64,
    frame_percent: f64,

    stm: SnakeTextureManager<'a>,
}

fn main() {

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let mut window = video_subsystem
        .window("snake.rs - SDL2 Driver", WIDTH_PIXELS, HEIGHT_PIXELS)
        .position(0, 0)
        .build()
        .unwrap();

    let mut display_mode = window.display_mode().unwrap();

    display_mode.format = sdl2::pixels::PixelFormatEnum::RGBA8888;

    window.set_display_mode(display_mode);

    let timer = sdl_context.timer().unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let stm = SnakeTextureManager::new(GAME_TO_PIXEL, CELL_MARGIN_PX, &texture_creator);

    // let mut event_pump = sdl_context.event_pump().unwrap();

    let mut ctx: SDLContext = SDLContext {
        canvas: canvas,
        event_pump: sdl_context.event_pump().unwrap(),
        timer: timer,
        timer_freq: 0,
        start_time: 0,
        last_frame_time: 0,
        last_tick_time: 0,
        frame_counter: 0,
        tick_counter: 0,
        frame_percent: 0.0,
        stm: stm,
    };

    ctx.last_frame_time = sdl2::TimerSubsystem::performance_counter(&ctx.timer);
    ctx.start_time = ctx.last_frame_time;
    ctx.timer_freq = sdl2::TimerSubsystem::performance_frequency(&ctx.timer);

    let mut game = GameState::new(WIDTH, HEIGHT);

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

        ctx.frame_percent = ((cur_time - ctx.last_tick_time) as f64) / std::time::Duration::as_nanos(&TICK_DURATION) as f64;

        if ctx.frame_percent >= 1.0 {

            ctx.frame_percent = ctx.frame_percent - 1.0;

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

fn rotate_rect(center: &(i32, i32), rect: &sdl2::rect::Rect, direction: &Direction) -> sdl2::rect::Rect {

    // rotation is CCW
    let matrix = direction.rotation_matrix();

    let corners: [(i32, i32); 4] = [
        (rect.x - center.0,          rect.y - center.1),
        (rect.x + rect.w - center.0, rect.y - center.1),
        (rect.x + rect.w - center.0, rect.y + rect.h - center.1),
        (rect.x - center.0,          rect.y + rect.h - center.1),
    ];

    // rotate and un-translate
    let rotated: Vec<_> = corners.iter().map(
        |p| (
            (p.0 * matrix.0.0) + (p.1 * matrix.0.1) + center.0,
            (p.0 * matrix.1.0) + (p.1 * matrix.1.1) + center.1,
        )
    ).collect();

    // find the rotated top left
    let pts: ((i32, i32), (i32, i32)) = match direction {
        Direction::Up    => (rotated[0], rotated[2]),
        Direction::Down  => (rotated[2], rotated[0]),
        Direction::Left  => (rotated[1], rotated[3]),
        Direction::Right => (rotated[3], rotated[1]),
    };

    let out = sdl2::rect::Rect::new (pts.0.0,
                                     pts.0.1,
                                     (pts.1.0 - pts.0.0) as u32,
                                     (pts.1.1 - pts.0.1) as u32);

    // println!("rotate {:?} to {:?}", rect, out);

    return out;
}

impl SDLContext<'_> {
    fn draw_food(&mut self, at: &(usize, usize)) {
        self.canvas.set_draw_color(FOOD_COLOR);
        let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(
            ((at.0 as u32 * GAME_TO_PIXEL) + CELL_MARGIN_PX) as i32,
            ((at.1 as u32 * GAME_TO_PIXEL) + CELL_MARGIN_PX) as i32,
            GAME_TO_PIXEL - (CELL_MARGIN_PX * 2),
            GAME_TO_PIXEL - (CELL_MARGIN_PX * 2),
        ));
    }

    fn draw(&mut self, game: &GameState) {

        // update background
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();

        self.canvas.set_draw_color(RED);
        let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(20, 100, 200, 50));

        self.canvas.pie(500, 500, 50, 0, 59, sdl2::pixels::Color::RGB(128, 128, 255));

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

        self.stm.draw_snake(self.frame_percent, &game, &mut self.canvas);

        self.canvas.present();

        if RATE_LIMITED {
            let cur_time: u64 = sdl2::TimerSubsystem::performance_counter(&self.timer);
            let frame_elapsed: u64 = cur_time - self.last_frame_time;
            let time_to_next_frame =
                FRAME_DURATION - std::time::Duration::from_secs(frame_elapsed / self.timer_freq);

            if time_to_next_frame > std::time::Duration::from_nanos(0) {
                std::thread::sleep(time_to_next_frame);
            }

            self.last_frame_time = cur_time;
        }
    }

    fn get_input(&mut self) -> InputType {
        for event in self.event_pump.poll_iter() {
            match event {

                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Q),
                    ..
                } => {
                    return InputType::Quit;
                }

                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Q),
                    ..
                } => {
                    return InputType::Quit;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Up),
                    ..
                } => {
                    return InputType::Up;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Right),
                    ..
                } => {
                    return InputType::Right;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Down),
                    ..
                } => {
                    return InputType::Down;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Left),
                    ..
                } => {
                    return InputType::Left;
                }
                _ => InputType::Nothing,
            };
        }
        return InputType::Nothing;
    }
}
