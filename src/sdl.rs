use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::gfx::primitives::DrawRenderer;
// use sdl2::render::Canvas;
// use sdl2::video::Window;
// use sdl2::EventPump;

// use snake::Coord;
// use snake::Direction;
// use snake::ItemType;
// use snake::StateTransition;

use std::time::Duration;

const WIDTH_PIXELS: u32 = 1200;
const HEIGHT_PIXELS: u32 = 800;

const GAME_TO_SCREEN_FACTOR: u32 = 50;

const CELL_MARGIN: u32 = 4;

const WIDTH: u32 = WIDTH_PIXELS / GAME_TO_SCREEN_FACTOR;
const HEIGHT: u32 = HEIGHT_PIXELS / GAME_TO_SCREEN_FACTOR;

const FRAMES_PER_SECOND: f64 = 60.0;
const FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000.0 / FRAMES_PER_SECOND) as u64);

const RATE_LIMITED: bool = true;

const TICKS_PER_SECOND: f64 = 1.0;
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
    frame_percent: f64,
}

trait SnakeGameRenderTrait {
    fn draw_food(&mut self, at: &(usize, usize));
    fn draw_snake(&mut self, at: &(usize, usize));
    fn draw_animated_snake(&mut self, game: &snake::GameState);
    fn transform_game_rect_to_raster(&self,
                                game: &snake::GameState,
                                x: i32,
                                y: i32,
                                w: i32,
                                h: i32,
                                direction: &snake::Direction) -> Rect;
    fn connect_snake_bits(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
    );
    fn draw_animated_snake_head(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
    );
    fn draw_animated_snake_bit(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
    );
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("snake.rs - SDL2 Driver", WIDTH_PIXELS, HEIGHT_PIXELS)
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
        frame_percent: 0.0,
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
            snake::StateTransition::Stop => break,
            _ => (),
        }

        let cur_time = sdl2::TimerSubsystem::performance_counter(&ctx.timer);

        ctx.frame_percent = ((cur_time - ctx.last_tick_time) as f64) / Duration::as_nanos(&TICK_DURATION) as f64;

        if ctx.frame_percent >= 1.0 {
            println!(
                "frames: {}; Tick FPS: {:.02}; Avg FPS: {:.02}",
                ctx.frame_counter,
                1e9 * (((ctx.frame_counter - last_tick_frame_number) as f64)
                    / ((cur_time - ctx.last_tick_time) as f64)),
                1e9 * ((ctx.frame_counter as f64) / ((cur_time - ctx.start_time) as f64)),
            );

            match game.update_state() {
                snake::StateTransition::Stop => break,
                _ => (),
            }

            last_tick_frame_number = ctx.frame_counter;
            ctx.tick_counter += 1;
            ctx.last_tick_time = cur_time;
        }

        ctx.frame_counter += 1;
    }
}

fn rotate_rect(game: &snake::GameState, rect: &Rect, direction: &snake::Direction) -> Rect {
    let xtranslate: i32 = rect.x + (rect.w / 2) as i32;
    let ytranslate: i32 = rect.y + (rect.h / 2) as i32;

    // rotation is CCW
    let matrix = direction.rotation_matrix();

    println!("rect: {:?}; translate: ({},{})", rect, xtranslate, ytranslate);

    let corners: [(i32, i32); 4] = [
        (rect.x - xtranslate,          rect.y - ytranslate),
        (rect.x + rect.w - xtranslate, rect.y - ytranslate),
        (rect.x + rect.w - xtranslate, rect.y + rect.h - ytranslate),
        (rect.x - xtranslate,          rect.y + rect.h - ytranslate),
    ];

    // for pt in corners {
    println!("corners: {:?}", corners);
    // }

    // rotate and un-translate
    let rotated: Vec<_> = corners.iter().map(
        |p| (
            (p.0 * matrix.0.0) + (p.1 * matrix.0.1) + xtranslate,
            (p.0 * matrix.1.0) + (p.1 * matrix.1.1) + ytranslate,
        )
    ).collect();

    println!("rotated: {:?}", rotated);

    // find the rotated top left
    let pts: ((i32, i32), (i32, i32)) = match direction {
        snake::Direction::Up    => (rotated[0], rotated[2]),
        snake::Direction::Down  => (rotated[2], rotated[0]),
        snake::Direction::Left  => (rotated[1], rotated[3]),
        snake::Direction::Right => (rotated[3], rotated[1]),
    };

    let out = Rect::new(pts.0.0, pts.0.1, -(pts.0.0 - pts.1.0) as u32, -(pts.0.1 - pts.1.1) as u32);

    println!("rotating {:?} to {:?}", rect, out);

    return out;
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

        let mut iter = game.get_snake().get_body().iter();

        if let Some(first) = iter.next() {

            self.draw_animated_snake_head(game, &first);

            for cur in iter {
                self.draw_animated_snake_bit(game, &cur);
                self.connect_snake_bits(game, &cur);
            }
        }
    }

    /*
     * given a point in the game, translate it to raster, rotated by direction, and with
     * partial_frame
     * */
    fn transform_game_rect_to_raster(&self,
                                game: &snake::GameState,
                                x: i32,
                                y: i32,
                                w: i32,
                                h: i32,
                                direction: &snake::Direction)
        -> Rect
    {
        return Rect::new(0,0,0,0);
    }

    fn draw_animated_snake_head(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
    ) {
        let pt: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());

        let partial: i32 = ((GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2)) as f64 * self.frame_percent) as i32;
        let one_minus_partial: i32 = ((GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2)) as f64 * (1.0 - self.frame_percent)) as i32;

        let WHOLE: i16 = (GAME_TO_SCREEN_FACTOR - (2 * CELL_MARGIN)) as i16;
        let HALF: i16 = WHOLE / 2;

        let adjust: (i16, i16, i16, i16) = match at.dir_next {
            Some(snake::Direction::Up) => (HALF, WHOLE, 179, 0),
            Some(snake::Direction::Down) => (HALF, 0, 0, 179),
            Some(snake::Direction::Right) => (0, HALF, 269, 90),
            Some(snake::Direction::Left) => (WHOLE, HALF, 90, 269),
            None => (0, 0, 0, 0),
        };

        self.canvas.filled_pie(
            (pt.0 as u32 * GAME_TO_SCREEN_FACTOR) as i16 + CELL_MARGIN as i16 + adjust.0,
            (pt.1 as u32 * GAME_TO_SCREEN_FACTOR) as i16 + CELL_MARGIN as i16 + adjust.1,
            ((GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2)) / 2) as i16,
            adjust.2,
            adjust.3,
            SNAKE_COLOR
        );
    }

    fn draw_animated_snake_bit(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
    ) {
        let pt: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());

        let rect = Rect::new(
                (pt.0 as u32 * GAME_TO_SCREEN_FACTOR) as i32 + CELL_MARGIN as i32,
                (pt.1 as u32 * GAME_TO_SCREEN_FACTOR) as i32 + CELL_MARGIN as i32,
                GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2),
                GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2),
        );

        if let Some(dir_next) = at.dir_next {
            if let Some(dir_prev) = at.dir_prev {
                if dir_next == dir_prev.get_opposite() {
                    self.canvas.set_draw_color(SNAKE_COLOR);
                    let _ = self.canvas.fill_rect(rect);
                } else {
                    println!("{:?} -> {:?}", dir_prev.get_opposite(), dir_next);
                    let WHOLE: i16 = GAME_TO_SCREEN_FACTOR as i16 - (CELL_MARGIN * 2) as i16;
                    let arc: (i16, i16, i16, i16) = match (dir_prev.get_opposite(), dir_next) {
                        (snake::Direction::Up, snake::Direction::Right) => (WHOLE, WHOLE, 180, 269), // OK
                        (snake::Direction::Up, snake::Direction::Left) => (0, WHOLE, 270, 359), // OK
                        (snake::Direction::Down, snake::Direction::Right) => (WHOLE, 0, 90, 179), // OK
                        (snake::Direction::Down, snake::Direction::Left) => (0, 0, 0, 89), // OK
                        (snake::Direction::Left, snake::Direction::Up) => (WHOLE, 0, 90, 179), // OK
                        (snake::Direction::Left, snake::Direction::Down) => (WHOLE, WHOLE, 180, 269), // OK
                        (snake::Direction::Right, snake::Direction::Up) => (0, 0, 0, 89), // OK
                        (snake::Direction::Right, snake::Direction::Down) => (0, WHOLE, 270, 359), // OK
                        _ => (0, 0, 0,0),
                    };

                    // draw a corner
                    self.canvas.set_draw_color(Color::RGB(255, 0, 0));
                    self.canvas.filled_pie(
                        rect.x as i16 + arc.0,
                        rect.y as i16 + arc.1,
                        (GAME_TO_SCREEN_FACTOR - (CELL_MARGIN * 2)) as i16,
                        arc.2, arc.3,
                        SNAKE_COLOR
                    );
                }
            } else {
                self.canvas.set_draw_color(SNAKE_COLOR);
                let _ = self.canvas.fill_rect(rect);
            }
        }

    }

    fn connect_snake_bits(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
    ) {
        self.canvas.set_draw_color(SNAKE_COLOR_DARK);

        let p = game.game_to_grid_tuple(&at.coord.as_tuple());

        let GF: i32 = GAME_TO_SCREEN_FACTOR as i32;
        let CM: i32 = CELL_MARGIN as i32;
        let CM2: u32 = CM as u32 * 2;

        let cx: i32 = p.0 as i32 * GF;
        let cy: i32 = p.1 as i32 * GF;

        if let Some(dir) = at.dir_next {
            let r = match dir {
                snake::Direction::Down => (cx + CM, cy - CM + GF as i32, GF as u32 - CM2, CM2),
                snake::Direction::Up => (cx + CM, cy - CM, GF as u32 - CM2, CM2),
                snake::Direction::Right => (cx - CM + GF as i32, cy + CM, CM2, GF as u32 - CM2),
                snake::Direction::Left => (cx - CM, cy + CM, CM2, GF as u32 - CM2),
            };

            let _ = self.canvas.fill_rect(Rect::new(r.0, r.1, r.2, r.3));
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

        self.canvas.set_draw_color(Color::RGB(255, 0, 0));
        let _ = self.canvas.fill_rect(Rect::new(20, 100, 200, 50));

        /*
        game logic down here
        */

        // render the grid
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                match &game.get_world()[y][x] {
                    snake::ItemType::Nothing => (),
                    snake::ItemType::Food => self.draw_food(&(x, y)),
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
