
use sdl2::gfx::primitives::DrawRenderer;

use snake::Direction;
use snake::CoordWithDirection;
use snake::GameState;

const SNAKE_COLOR_LIGHT: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 200, 50, 255);
const SNAKE_COLOR_DARK: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 150, 60, 255);
const RED: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(255, 0, 0, 255);
const BLUE: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 255, 255);
const BLACK: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 0, 255);
const WHITE: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(255, 255, 255, 255);

pub struct SnakeTextureManager<'a> {
    head: sdl2::render::Texture<'a>,
    bits: std::collections::LinkedList<sdl2::surface::Surface<'a>>,

    tile_dimension: u32,
    tile_margin: u32,

    snake_width: i16,
    half_snake_width_f64: f64,
}

impl<'a> SnakeTextureManager<'a> {
    pub fn new(tile_dimension: u32,
               tile_margin: u32,
               texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>)
        -> Self
    {

        let snake_width: i16 = (tile_dimension - (2 * tile_margin)) as i16;

        return SnakeTextureManager {
            head: Self::create_head_texture(tile_dimension, snake_width, texture_creator),
            bits: std::collections::LinkedList::new(),
            // tail: sdl2::surface::Surface::new(tile_dimension, tile_dimension, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap(),

            tile_dimension: tile_dimension,
            tile_margin: tile_margin,

            snake_width: snake_width,
            half_snake_width_f64: snake_width as f64 / 2.0,
        };
    }

    fn get_direction_angle(direction: &snake::Direction) -> f64 {
        match direction {
            Direction::Up => 270.0,
            Direction::Right => 0.0,
            Direction::Down => 90.0,
            Direction::Left => 180.0,
        }
    }

    // draw the head facing right (angle 0) and no partial/adjustment
    fn create_head_texture(tile_dimension: u32,
                           snake_width: i16,
                           texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>)
        -> sdl2::render::Texture
    {
        let head: sdl2::surface::Surface = sdl2::surface::Surface::new(tile_dimension,
                                                                       tile_dimension,
                                                                       sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();

        let mut head_canvas = head.into_canvas().unwrap();

        // transparent background
        // head_canvas.set_draw_color(sdl2::pixels::Color::RGBA(255,0,0,128));
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

        head_canvas.into_surface().as_texture(&texture_creator).unwrap()
    }

    pub fn draw_snake(&mut self,
                      frame_percent: f64,
                      game: &snake::GameState,
                      canvas: &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        canvas.set_draw_color(SNAKE_COLOR_LIGHT);

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
        let pt_: (usize, usize) = game.game_to_grid_tuple(&at.coord.as_tuple());

        let pt: (i32, i32) = (pt_.0 as i32 * self.tile_dimension as i32,
                              pt_.1 as i32 * self.tile_dimension as i32);

        // calculate angle
        let target_direction = at.dir_next;

        let incoming_direction = match at.dir_prev {
            Some(d) => d.get_opposite(),
            _       => target_direction,
        };

        let target_angle_deg: f64 = Self::get_direction_angle(&target_direction);
        let incoming_angle_deg: f64 = Self::get_direction_angle(&incoming_direction);

        // angle of forward direction
        let forward_angle_deg: f64 = match (incoming_direction, target_direction) {
            // rotate special cases in correct direction
            (snake::Direction::Up, snake::Direction::Right)
                => incoming_angle_deg + ((360.0 - incoming_angle_deg) * frame_percent),

            (snake::Direction::Right, snake::Direction::Up)
                => 360.0 + ((target_angle_deg - 360.0) * frame_percent),

            // general case rotates in correct direction
            _   => incoming_angle_deg + ((target_angle_deg - incoming_angle_deg) * frame_percent),
        };

        // find target root point in grid
        let (tx, ty) = (frame_percent * self.half_snake_width_f64 * forward_angle_deg.to_radians().cos(),
                        frame_percent * self.half_snake_width_f64 * forward_angle_deg.to_radians().sin());

        // translate rotated surface root point to grid
        let (sx, sy) = (pt.0 + tx as i32, pt.1 + ty as i32);

        println!("%:{}, in:{}, fwd:{:.1}, tgt:{}, t:({:.1},{:.1}), s:({},{})",
                 frame_percent,
                 incoming_angle_deg,
                 forward_angle_deg,
                 target_angle_deg,
                 tx,ty,
                 sx,sy);

        canvas.copy_ex(&self.head,        // texture
                       None,              // src rect -- None = entire texture
                       sdl2::rect::Rect::new(sx + self.tile_margin as i32,
                                             sy + self.tile_margin as i32,
                                             self.snake_width as u32,
                                             self.snake_width as u32), // dst rect
                       forward_angle_deg, // angle of rotation
                       None,              // center for rotation -- None = dst (or src if dst None)
                       false,             // flip_horizontal
                       false);            // flip_vertical

        // try to draw a box around the rotated one
        let rect = ();

        let rect = sdl2::rect::Rect::new (
            pt.0 as i32 + self.tile_margin as i32,
            pt.1 as i32 + self.tile_margin as i32,
            self.snake_width as u32,
            self.snake_width as u32,
        );

        if incoming_angle_deg == target_angle_deg {
            // not rotating
            canvas.set_draw_color(BLUE);
        } else {
            // should be rotating
            canvas.set_draw_color(RED);
        }
        let _ = canvas.draw_rect(rect);

        let gc = (pt.0 as i16 + (self.tile_dimension as f64 / 2.0) as i16,
                  pt.1 as i16 + (self.tile_dimension as f64 / 2.0) as i16);

        // draw the current grid center
        let _ = canvas.filled_circle(gc.0, gc.1, 3, BLACK);

        // draw dots at incoming and exiting points
        let incoming_pt = (
            gc.0 as i32 + tx as i32,
            gc.1 as i32 + ty as i32,
        );
        let _ = canvas.filled_circle(incoming_pt.0 as i16, incoming_pt.1 as i16, 3, BLUE);

        // draw a nice arc from incoming angle to target angle
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
            let _ = canvas.draw_rect(rotated);
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
            let _ = canvas.draw_rect(rotated);
            // self.canvas.set_draw_color(RED); let _ = self.canvas.draw_rect(rotated);
        }
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
