
// provides pie and filled_pie
use sdl2::gfx::primitives::DrawRenderer;

const WIDTH_PIXELS: u32 = 1200;
const HEIGHT_PIXELS: u32 = 800;

const GAME_TO_PIXEL: u32 = 50;

const CELL_MARGIN_PX: u32 = 4;

const WIDTH: u32 = WIDTH_PIXELS / GAME_TO_PIXEL;
const HEIGHT: u32 = HEIGHT_PIXELS / GAME_TO_PIXEL;

const FRAMES_PER_SECOND: f64 = 30.0;
const FRAME_DURATION: std::time::Duration = std::time::Duration::from_nanos((1_000_000_000.0 / FRAMES_PER_SECOND) as u64);

const RATE_LIMITED: bool = true;

const TICKS_PER_SECOND: f64 = 1.0;
const TICK_DURATION: std::time::Duration = std::time::Duration::from_nanos((1_000_000_000.0 / TICKS_PER_SECOND) as u64);

const FOOD_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(200, 200, 20);
const SNAKE_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 200, 50);
const SNAKE_COLOR_DARK: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 150, 60);
const RED: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 0, 0);
const BLUE: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 255);

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

    snake_direction_last: snake::Direction,
    snake_position_last: i16,
    radius_px: i16,
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
                                direction: &snake::Direction) -> sdl2::rect::Rect;

    fn draw_animated_snake_head(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
        next: Option<&snake::CoordWithDirection>,
    );

    fn draw_animated_snake_bit(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
        prev: &snake::CoordWithDirection,
        next: Option<&snake::CoordWithDirection>,
    );

    fn draw_snake_body_angle(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
        angle: i16
    );

    fn draw_bounding_box(&mut self, pt_px: &(i32, i32), direction: &snake::Direction, partial_px: i32);
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
        canvas_here.set_draw_color(sdl2::pixels::Color::RGB(0, 255, 255));
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
        snake_direction_last: snake::Direction::Up,
        snake_position_last: 0,
        radius_px: ((GAME_TO_PIXEL - (CELL_MARGIN_PX * 2)) / 2) as i16,
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

fn rotate_rect(center: &(i32, i32), rect: &sdl2::rect::Rect, direction: &snake::Direction) -> sdl2::rect::Rect {

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
        snake::Direction::Up    => (rotated[0], rotated[2]),
        snake::Direction::Down  => (rotated[2], rotated[0]),
        snake::Direction::Left  => (rotated[1], rotated[3]),
        snake::Direction::Right => (rotated[3], rotated[1]),
    };

    let out = sdl2::rect::Rect::new (pts.0.0,
                                     pts.0.1,
                                     (pts.1.0 - pts.0.0) as u32,
                                     (pts.1.1 - pts.0.1) as u32);

    // println!("rotate {:?} to {:?}", rect, out);

    return out;
}

impl SnakeGameRenderTrait for SDLContext<'_> {

    fn draw_food(&mut self, at: &(usize, usize)) {
        self.canvas.set_draw_color(FOOD_COLOR);
        let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(
            ((at.0 as u32 * GAME_TO_PIXEL) + CELL_MARGIN_PX) as i32,
            ((at.1 as u32 * GAME_TO_PIXEL) + CELL_MARGIN_PX) as i32,
            GAME_TO_PIXEL - (CELL_MARGIN_PX * 2),
            GAME_TO_PIXEL - (CELL_MARGIN_PX * 2),
        ));
    }

    fn draw_snake(&mut self, at: &(usize, usize)) {
        self.canvas.set_draw_color(SNAKE_COLOR);
        let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(
            ((at.0 as u32 * GAME_TO_PIXEL) + CELL_MARGIN_PX) as i32,
            ((at.1 as u32 * GAME_TO_PIXEL) + CELL_MARGIN_PX) as i32,
            GAME_TO_PIXEL - (CELL_MARGIN_PX * 2),
            GAME_TO_PIXEL - (CELL_MARGIN_PX * 2),
        ));
    }

    /*
     * draw the snake by from the head to the tail. each piece of snake drawn includes a reference
     * to the next (further back) piece to calculate the partial corner when needed.
     * */
    fn draw_animated_snake(&mut self, game: &snake::GameState) {
        self.canvas.set_draw_color(SNAKE_COLOR);

        let mut prev: Option<&snake::CoordWithDirection> = None;

        let mut iter = game.get_snake().get_body().iter();

        if let Some(head) = iter.next() {
            let mut prev = head;
            let next_bit = iter.next();

            self.draw_animated_snake_head(game, head, next_bit);

            match next_bit {
                Some(bit) => {
                    let mut cur = bit;
                    for following in iter {
                        // middle
                        self.draw_animated_snake_bit(game, cur, prev, Some(&following));
                        prev = cur;
                        cur = following;
                    }
                    // tail
                    self.draw_animated_snake_bit(game, cur, prev, None);
                },
                None => (),
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
        -> sdl2::rect::Rect
    {
        return sdl2::rect::Rect::new(0,0,0,0);
    }

    fn draw_animated_snake_head(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
        next: Option<&snake::CoordWithDirection>,
    ) {
        let pt: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());
        let pt_px: (i32, i32) = (pt.0 as i32 * GAME_TO_PIXEL as i32, pt.1 as i32 * GAME_TO_PIXEL as i32);

        let partial_px: i32 = (GAME_TO_PIXEL as f64 * self.frame_percent) as i32;

        let WHOLE: i16 = GAME_TO_PIXEL as i16;
        let HALF: i16 = WHOLE / 2;

        let adjust: (i16, i16, i16, i16) = match at.dir_next {
            snake::Direction::Up => (HALF, WHOLE - partial_px as i16 + self.radius_px, 179, 0),
            snake::Direction::Down => (HALF, partial_px as i16 - self.radius_px, 0, 179),
            snake::Direction::Right => (partial_px as i16 - self.radius_px, HALF, 269, 90),
            snake::Direction::Left => (WHOLE - partial_px as i16 + self.radius_px, HALF, 90, 269),
        };

        let center = (
            (pt.0 as u32 * GAME_TO_PIXEL) as i16 + adjust.0,
            (pt.1 as u32 * GAME_TO_PIXEL) as i16 + adjust.1,
        );

        let snake_position_last = match at.dir_next {
            snake::Direction::Up | snake::Direction::Down => center.1,
            snake::Direction::Left | snake::Direction::Right => center.0,
        };

        if self.snake_direction_last == at.dir_next {
            if self.snake_position_last == snake_position_last {
                // return false;
            }
        } else {
            self.snake_direction_last = at.dir_next;
        }

        self.snake_position_last = snake_position_last;

        self.canvas.filled_pie(
            center.0,
            center.1,
            self.radius_px,
            adjust.2,
            adjust.3,
            SNAKE_COLOR
        );

        self.draw_bounding_box(&pt_px, &at.dir_next, partial_px);
    }

    fn draw_bounding_box(&mut self,
                         pt_px: &(i32, i32),
                         direction: &snake::Direction,
                         partial_px: i32)
    {
        let adj: i32 = 0; // self.radius_px as i32; // + partial_px;

        let bounding_partial: (i32, i32) = match direction {
            snake::Direction::Up => (0, adj),
            snake::Direction::Down => (0, -adj),
            snake::Direction::Left => (adj, 0),
            snake::Direction::Right => (-adj, 0),
        };

        let bounding = sdl2::rect::Rect::new (
            pt_px.0 as i32 + CELL_MARGIN_PX as i32 + bounding_partial.0,
            pt_px.1 as i32 + CELL_MARGIN_PX as i32 + bounding_partial.1,
            (GAME_TO_PIXEL - (2 * CELL_MARGIN_PX)) as u32,
            (GAME_TO_PIXEL - (2 * CELL_MARGIN_PX)) as u32,
        );

        self.canvas.set_draw_color(RED);
        let _ = self.canvas.draw_rect(bounding);
    }

    fn draw_snake_body_angle(
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
        angle: i16
    )
    {
        angle;
        /*
        self.draw_snake_body_angle(game, at, angle);
        let one_minus_partial: i32 = (GAME_TO_PIXEL as f64 * (1.0 - self.frame_percent)) as i32;
        let mut one_minus_partial_block: u32 = one_minus_partial as u32;

        let WHOLE: i16 = GAME_TO_PIXEL as i16 - (CELL_MARGIN_PX * 2) as i16;
        let arc: (i16, i16, i16, i16) = match (at.dir_prev.expect("yes").get_opposite(), at.dir_next) {
            (snake::Direction::Up,    snake::Direction::Right) => (WHOLE, WHOLE, 180, 269), // OK
            (snake::Direction::Up,    snake::Direction::Left)  => (0, WHOLE, 270, 359), // OK
            (snake::Direction::Down,  snake::Direction::Right) => (WHOLE, 0, 90, 179), // OK
            (snake::Direction::Down,  snake::Direction::Left)  => (0, 0, 0, 89), // OK
            (snake::Direction::Left,  snake::Direction::Up)    => (WHOLE, 0, 90, 179), // OK
            (snake::Direction::Left,  snake::Direction::Down)  => (WHOLE, WHOLE, 180, 269), // OK
            (snake::Direction::Right, snake::Direction::Up)    => (0, 0, 0, 89), // OK
            (snake::Direction::Right, snake::Direction::Down)  => (0, WHOLE, 270, 359), // OK
            _ => (0, 0, 0,0),
        };

        // draw a corner
        self.canvas.filled_pie(
            (pt.0 as i16 * GAME_TO_PIXEL as i16) + CELL_MARGIN_PX as i16 + arc.0,
            (pt.1 as i16 * GAME_TO_PIXEL as i16) + CELL_MARGIN_PX as i16 + arc.1,
            (GAME_TO_PIXEL - (CELL_MARGIN_PX * 2)) as i16,
            arc.2,
            arc.3,
            SNAKE_COLOR
        );
        self.canvas.arc(
            (pt.0 as i16 * GAME_TO_PIXEL as i16) + CELL_MARGIN_PX as i16 + arc.0,
            (pt.1 as i16 * GAME_TO_PIXEL as i16) + CELL_MARGIN_PX as i16 + arc.1,
            (GAME_TO_PIXEL - (CELL_MARGIN_PX * 2)) as i16,
            arc.2,
            arc.3,
            RED,
        );
        */
    }

    fn draw_animated_snake_bit (
        &mut self,
        game: &snake::GameState,
        at: &snake::CoordWithDirection,
        prev: &snake::CoordWithDirection,
        next: Option<&snake::CoordWithDirection>,
    ) {

        // top-right of target square
        let pt: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());
        let pt_px: (i32, i32) = (pt.0 as i32 * GAME_TO_PIXEL as i32, pt.1 as i32 * GAME_TO_PIXEL as i32);

        // center of target square
        let c_px: (i32, i32) = (
            pt_px.0 as i32 + (GAME_TO_PIXEL / 2) as i32,
            pt_px.1 as i32 + (GAME_TO_PIXEL / 2) as i32,
        );

        // offset
        let partial_px: i32 = (GAME_TO_PIXEL as f64 * self.frame_percent) as i32;
        // extent
        let one_minus_partial_px: i32 = (GAME_TO_PIXEL as f64 * (1.0 - self.frame_percent)) as i32;

        let curve_from_start = prev.dir_next != at.dir_next;
        let curve_to_end = next.is_some() && at.dir_next != next.unwrap().dir_next;

        // pre-curve
        if false && partial_px < self.radius_px as i32 {
            // up
            let rect = sdl2::rect::Rect::new (
                pt_px.0 as i32 + CELL_MARGIN_PX as i32,
                pt_px.1 as i32 - partial_px + self.radius_px as i32,
                (GAME_TO_PIXEL - (2 * CELL_MARGIN_PX)) as u32,
                partial_px as u32,
            );
            let rotated = rotate_rect(&c_px, &rect, &at.dir_next);
            self.canvas.set_draw_color(SNAKE_COLOR);
            let _ = self.canvas.fill_rect(rotated);
        }

        if curve_from_start {
        } else if true || partial_px >= self.radius_px as i32 {
            // go from radius to partial
            let rect = sdl2::rect::Rect::new (
                pt_px.0 as i32 + CELL_MARGIN_PX as i32,
                pt_px.1 as i32 - partial_px + self.radius_px as i32,
                (GAME_TO_PIXEL - (2 * CELL_MARGIN_PX)) as u32,
                partial_px as u32,
            );
            let rotated = rotate_rect(&c_px, &rect, &at.dir_next);
            self.canvas.set_draw_color(SNAKE_COLOR);
            let _ = self.canvas.fill_rect(rotated);
        }

        if curve_to_end {
        } else {
        }

        // post-curve
        if one_minus_partial_px > 0 {
            // up
            let rect = sdl2::rect::Rect::new (
                pt_px.0 as i32 + CELL_MARGIN_PX as i32,
                pt_px.1 as i32 + self.radius_px as i32,
                (GAME_TO_PIXEL - (2 * CELL_MARGIN_PX)) as u32,
                one_minus_partial_px as u32,
            );
            let rotated = rotate_rect(&c_px, &rect, &at.dir_next);
            self.canvas.set_draw_color(SNAKE_COLOR);
            let _ = self.canvas.fill_rect(rotated);
            // self.canvas.set_draw_color(RED); let _ = self.canvas.draw_rect(rotated);
        }

        // self.draw_bounding_box(&pt_px, &at.dir_next, partial_px);

        /*
        let block: u32 = GAME_TO_PIXEL - (CELL_MARGIN_PX * 2);
        let mut partial_block: u32 = partial as u32;

        let mut pos_adjust: i32 = block as i32;

        let mut has_arc = true;

        let mut full_block: bool = at.dir_prev == None || at.dir_prev == Some(at.dir_next.get_opposite());

        if full_block {
            has_arc = false;
            partial_block = block;
            pos_adjust = partial;
        }

        let adjust: (i32, i32, u32, u32) = match at.dir_next {
            snake::Direction::Up => (0, -partial + self.radius_px as i32, block, partial_block), // ok
            snake::Direction::Down => (0, pos_adjust - self.radius_px as i32, block, partial_block), // ok
            snake::Direction::Right => (pos_adjust - self.radius_px as i32, 0, partial_block, block), // ok
            snake::Direction::Left => (-partial + self.radius_px as i32, 0, partial_block, block), // ok
        };

        let rect = sdl2::rect::Rect::new (
            (pt_px.0 as u32 * GAME_TO_PIXEL) as i32 + CELL_MARGIN_PX as i32 + adjust.0,
            (pt_px.1 as u32 * GAME_TO_PIXEL) as i32 + CELL_MARGIN_PX as i32 + adjust.1,
            adjust.2,
            adjust.3,
        );

        if adjust.2 > 0 && adjust.3 > 0 {
            self.canvas.set_draw_color(SNAKE_COLOR);
            let _ = self.canvas.fill_rect(rect);
        }

        // part after curve
        if curve_to_end {
            self.canvas.set_draw_color(RED);
            let _ = self.canvas.draw_rect(rect);
        }

        // part before curve
        if curve_from_start {
            self.canvas.set_draw_color(BLUE);
            let _ = self.canvas.draw_rect(rect);
        }
        */
    }
}

impl SDLContext<'_> {
    fn draw(&mut self, game: &snake::GameState) {

        // update background
        self.color_index = (self.color_index + 1) % 255;
        // self.canvas.set_draw_color(sdl2::pixels::Color::RGB(self.color_index, 64, 255 - self.color_index));
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();

        self.canvas.set_draw_color(RED);
        let _ = self.canvas.fill_rect(sdl2::rect::Rect::new(20, 100, 200, 50));

        self.canvas.pie(500, 500, 50, 0, 59, sdl2::pixels::Color::RGB(128, 128, 255));

        self.draw_food(&(0,0));
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
                FRAME_DURATION - std::time::Duration::from_secs(frame_elapsed / self.timer_freq);

            if time_to_next_frame > std::time::Duration::from_nanos(0) {
                std::thread::sleep(time_to_next_frame);
            }

            self.last_frame_time = cur_time;
        }
    }

    fn get_input(&mut self) -> snake::InputType {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    return snake::InputType::Quit;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Q),
                    ..
                } => {
                    return snake::InputType::Quit;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Up),
                    ..
                } => {
                    return snake::InputType::Up;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Right),
                    ..
                } => {
                    return snake::InputType::Right;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Down),
                    ..
                } => {
                    return snake::InputType::Down;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Left),
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
