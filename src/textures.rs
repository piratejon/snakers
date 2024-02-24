
use sdl2::gfx::primitives::DrawRenderer;

use sdl2::gfx::rotozoom::RotozoomSurface;

use snake::Direction;
use snake::CoordWithDirection;
use snake::GameState;

const SNAKE_COLOR_LIGHT: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 200, 50, 255);
const SNAKE_COLOR_DARK: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 150, 60, 255);

pub struct SnakeTextureManager<'a> {
    head: sdl2::surface::Surface<'a>,
    bits: std::collections::LinkedList<sdl2::surface::Surface<'a>>,
    tail: sdl2::surface::Surface<'a>,

    tile_dimension: u32,
    tile_margin: u32,

    snake_width: i16,
}

impl<'a> SnakeTextureManager<'a> {
    pub fn new(tile_dimension: u32, tile_margin: u32) -> Self {

        let snake_width: i16 = (tile_dimension - (2 * tile_margin)) as i16;

        let head: sdl2::surface::Surface = *Self::create_head_surface(tile_dimension, snake_width);

        return SnakeTextureManager {
            head: head,
            bits: std::collections::LinkedList::new(),
            tail: sdl2::surface::Surface::new(tile_dimension, tile_dimension, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap(),
            tile_dimension: tile_dimension,
            tile_margin: tile_margin,
            snake_width: snake_width,
        };
    }

    fn get_direction_angle(&snake::Direction) -> i16 {
        match self {
            Direction::Up => 270,
            Direction::Right => 0,
            Direction::Down => 90,
            Direction::Left => 180,
        }
    }

    // draw the head facing right (angle 0) and no partial/adjustment
    fn create_head_surface(tile_dimension: u32, snake_width: i16) -> Box<sdl2::surface::Surface<'a>> {

        let mut head: sdl2::surface::Surface = sdl2::surface::Surface::new(tile_dimension, tile_dimension, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();

        let mut head_canvas = head.into_canvas().unwrap();

        // transparent background
        head_canvas.set_draw_color(sdl2::pixels::Color::RGBA(0,0,0,0));
        head_canvas.clear();

        // draw a filled pie slice counter-clockwise
        let _ = head_canvas.filled_pie(
            0,                         // left edge
            tile_dimension as i16 / 2, // halfway down
            snake_width / 2,           // radius
            270,                       // bottom
            90,                        // top
            SNAKE_COLOR_LIGHT
        );

        Box::new(head_canvas.into_surface())
    }

    pub fn draw_snake(&mut self,
                      frame_percent: f64,
                      game: &snake::GameState,
                      canvas: &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        canvas.set_draw_color(SNAKE_COLOR_LIGHT);

        let mut prev: Option<&CoordWithDirection> = None;

        let mut iter = game.get_snake().get_body().iter();

        if let Some(head) = iter.next() {
            let mut prev = head;
            let next_bit = iter.next();

            self.draw_animated_snake_head(frame_percent, game, head, next_bit, canvas);

            match next_bit {
                Some(bit) => {
                    let mut cur = bit;
                    for following in iter {
                        // middle
                        self.draw_animated_snake_bit(frame_percent, game, cur, prev, Some(&following), canvas);
                        prev = cur;
                        cur = following;
                    }
                    // tail
                    self.draw_animated_snake_bit(frame_percent, game, cur, prev, None, canvas);
                },
                None => (),
            }
        }
    }

    /*
     * head is drawn lagged by the radius in the direction it came from so that it never exceeds
     * its box in the forward direction (otherwise it looks like a collision when there isn't one).
     *
     * when the snake changes direction:
     *  * the head gets a frame_percent angle along prevbit's nextdir to the new direction
     *  * the prevbit gets the same angle on its leading "half" (whatever portion has entered the
     *  new grid). the back "half" follows the same logic for changing direction.
     * */
    fn draw_animated_snake_head(&mut self,
                                frame_percent: f64,
                                game: &GameState,
                                at: &CoordWithDirection,
                                prev: Option<&CoordWithDirection>,
                                canvas: &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        let pt: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());
        let pt_px: (i32, i32) = (pt.0 as i32 * self.tile_dimension as i32, pt.1 as i32 * self.tile_dimension as i32);

        let partial_px: i32 = (self.tile_dimension as f64 * frame_percent) as i32;

        // calculate angle
        let target_angle = Self::get_direction_angle(at.dir_next);
        let reverse_angle = match prev {
            Some(d) => Self::get_direction_angle(d),
            _ => target_angle
        };

        let mut forward_angle = reverse_angle as f64 + ((target_angle - reverse_angle) as f64 * frame_percent);

        // clamp the angle
        while forward_angle >= 360.0 {
            forward_angle = forward_angle - 360.0;
        }
        while forward_angle < 0.0 {
            forward_angle = forward_angle + 360.0;
        }

        // rotate it
        let rotated_surface = self.head.rotozoom(forward_angle as f64,
                                                 1.0,    // zoom
                                                 false); // anti-aliasing

        // calculate the top left of the where to blit from in the rotated frame
        // root of head is (r sin t, r cos t).
        // root should be translated to target position
        // target position is

        let WHOLE: i16 = self.tile_dimension as i16;
        let HALF: i16 = WHOLE / 2;

        let adjust: (i16, i16, i16, i16) = match at.dir_next {
            Direction::Up => (HALF, WHOLE - partial_px as i16 + (self.snake_width / 2), 179, 0),
            Direction::Down => (HALF, partial_px as i16 - (self.snake_width / 2), 0, 179),
            Direction::Right => (partial_px as i16 - (self.snake_width / 2), HALF, 269, 90),
            Direction::Left => (WHOLE - partial_px as i16 + (self.snake_width / 2), HALF, 90, 269),
        };

        let center = (
            (pt.0 as u32 * self.tile_dimension) as i16 + adjust.0,
            (pt.1 as u32 * self.tile_dimension) as i16 + adjust.1,
        );
    }

    fn draw_animated_snake_bit (&mut self,
                                frame_percent: f64,
                                game: &GameState,
                                at: &CoordWithDirection,
                                prev: &CoordWithDirection,
                                next: Option<&CoordWithDirection>,
                                canvas: &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        // top-right of target square
        let pt: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());
        let pt_px: (i32, i32) = (pt.0 as i32 * self.tile_dimension as i32, pt.1 as i32 * self.tile_dimension as i32);

        // center of target square
        let c_px: (i32, i32) = (
            pt_px.0 as i32 + (self.tile_dimension / 2) as i32,
            pt_px.1 as i32 + (self.tile_dimension / 2) as i32,
        );

        // offset
        let partial_px: i32 = (self.tile_dimension as f64 * frame_percent) as i32;
        // extent
        let one_minus_partial_px: i32 = (self.tile_dimension as f64 * (1.0 - frame_percent)) as i32;

        let curve_from_start = prev.dir_next != at.dir_next;
        let curve_to_end = next.is_some() && at.dir_next != next.unwrap().dir_next;

        // pre-curve
        if false && partial_px < (self.snake_width / 2) as i32 {
            // up
            let rect = sdl2::rect::Rect::new (
                pt_px.0 as i32 + self.tile_margin as i32,
                pt_px.1 as i32 - partial_px + (self.snake_width / 2) as i32,
                self.snake_width as u32,
                partial_px as u32,
            );
            let rotated = rotate_rect(&c_px, &rect, &at.dir_next);
            canvas.set_draw_color(SNAKE_COLOR_LIGHT);
            let _ = canvas.fill_rect(rotated);
        }

        if curve_from_start {
        } else if true || partial_px >= (self.snake_width / 2) as i32 {
            // go from radius to partial
            let rect = sdl2::rect::Rect::new (
                pt_px.0 as i32 + self.tile_margin as i32,
                pt_px.1 as i32 - partial_px + (self.snake_width / 2) as i32,
                self.snake_width as u32,
                partial_px as u32,
            );
            let rotated = rotate_rect(&c_px, &rect, &at.dir_next);
            canvas.set_draw_color(SNAKE_COLOR_LIGHT);
            let _ = canvas.fill_rect(rotated);
        }

        if curve_to_end {
        } else {
        }

        // post-curve
        if one_minus_partial_px > 0 {
            // up
            let rect = sdl2::rect::Rect::new (
                pt_px.0 as i32 + self.tile_margin as i32,
                pt_px.1 as i32 + (self.snake_width / 2) as i32,
                self.snake_width as u32,
                one_minus_partial_px as u32,
            );
            let rotated = rotate_rect(&c_px, &rect, &at.dir_next);
            canvas.set_draw_color(SNAKE_COLOR_LIGHT);
            let _ = canvas.fill_rect(rotated);
            // self.canvas.set_draw_color(RED); let _ = self.canvas.draw_rect(rotated);
        }

        // self.draw_bounding_box(&pt_px, &at.dir_next, partial_px);

        /*
        let block: u32 = self.tile_dimension - (self.tile_margin * 2);
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
            Direction::Up => (0, -partial + self.radius_px as i32, block, partial_block), // ok
            Direction::Down => (0, pos_adjust - self.radius_px as i32, block, partial_block), // ok
            Direction::Right => (pos_adjust - self.radius_px as i32, 0, partial_block, block), // ok
            Direction::Left => (-partial + self.radius_px as i32, 0, partial_block, block), // ok
        };

        let rect = sdl2::rect::Rect::new (
            (pt_px.0 as u32 * self.tile_dimension) as i32 + self.tile_margin as i32 + adjust.0,
            (pt_px.1 as u32 * self.tile_dimension) as i32 + self.tile_margin as i32 + adjust.1,
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
