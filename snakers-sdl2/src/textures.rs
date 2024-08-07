
use sdl2::gfx::primitives::DrawRenderer;

use snakers::direction::Direction;
use snakers::game::CoordWithDirection;
use snakers::game::GameState;

const SNAKE_COLOR_LIGHT: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 200, 50, 255);
const SNAKE_COLOR_DARK: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 150, 60, 255);
const RED: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(255, 0, 0, 255);
const BLUE: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 255, 255);
const BLACK: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(0, 0, 0, 255);
const WHITE: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(255, 255, 255, 255);
const YELLOW: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(255, 255, 0, 255);
const ORANGE: sdl2::pixels::Color = sdl2::pixels::Color::RGBA(255, 128, 0, 255);

pub struct SnakeTextureManager<'a> {
    head: sdl2::render::Texture<'a>,
    // bits: std::collections::LinkedList<sdl2::surface::Surface<'a>>,
    bit: sdl2::render::Texture<'a>,

    tile_dimension: u32,
    tile_margin: u32,

    snake_width: i16,
    half_snake_width_f64: f64,
    half_snake_width_normalized: f64,
}

#[derive(PartialEq)]
enum DirectionKind {
    Straight,
    Clockwise,
    Anticlockwise,
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
            bit: Self::create_body_texture(tile_dimension, snake_width, texture_creator),
                // std::collections::LinkedList::new(),
            // tail: sdl2::surface::Surface::new(tile_dimension, tile_dimension, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap(),

            tile_dimension: tile_dimension,
            tile_margin: tile_margin,

            snake_width: snake_width,
            half_snake_width_f64: snake_width as f64 / 2.0,
            half_snake_width_normalized: (snake_width as f64 / 2.0) / tile_dimension as f64,
        };
    }

    fn get_direction_angle(direction: &Direction) -> f64 {
        match direction {
            Direction::Up => 270.0,
            Direction::Right => 0.0,
            Direction::Down => 90.0,
            Direction::Left => 180.0,
        }
    }

    // draw a body textrue facing right (angle 0) and no partial/adjustment
    fn create_body_texture(tile_dimension: u32,
                           snake_width: i16,
                           texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>)
        -> sdl2::render::Texture
    {
        let body: sdl2::surface::Surface = sdl2::surface::Surface::new(tile_dimension,
                                                                       tile_dimension,
                                                                       sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();

        let mut body_canvas = body.into_canvas().unwrap();

        // transparent background
        // body_canvas.set_draw_color(sdl2::pixels::Color::RGBA(255,0,0,128));
        body_canvas.set_draw_color(sdl2::pixels::Color::RGBA(0,0,0,0));
        body_canvas.clear();

        body_canvas.set_draw_color(SNAKE_COLOR_LIGHT);

        body_canvas.fill_rect(
            sdl2::rect::Rect::new(
                0,
                ((tile_dimension - snake_width as u32) / 2) as i32,
                tile_dimension as u32,
                snake_width as u32
            ),
        );

        let _ = body_canvas.filled_pie(
            0,                         // left edge
            tile_dimension as i16 / 2, // halfway down
            snake_width / 2,           // radius
            270,                       // bottom
            90,                        // top
            BLUE
        );

        let _ = body_canvas.filled_circle(tile_dimension as i16 / 3, snake_width / 3, 3, RED);

        body_canvas.into_surface().as_texture(&texture_creator).unwrap()
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

    /*
     * src_offset, dst_offset, and size are all along the direction from node to dst_node
     * */
    fn draw_partial_snake_bit(&self,
                              node:       &CoordWithDirection,
                              src_offset: f64,
                              size:       f64,
                              dst_offset: f64,
                              dst_node:   &CoordWithDirection,
                              game:       &GameState,
                              canvas:     &mut sdl2::render::Canvas<sdl2::video::Window>,
                              color:      sdl2::pixels::Color)
    {
        let size_px = (size * self.tile_dimension as f64) as u32;

        let src_rect = sdl2::rect::Rect::new(
            (src_offset * self.tile_dimension as f64) as i32,
            0,
            size_px,
            self.tile_dimension);

        let is_corner = node.dir_prev != node.dir_next.get_opposite();

        if is_corner {
        } else {
            let pt: (usize, usize) = game.game_to_grid_tuple(&dst_node.coord.as_tuple());

            let pt: (i32, i32) = (pt.0 as i32 * self.tile_dimension as i32,
                                  pt.1 as i32 * self.tile_dimension as i32);

            let dst_rect = sdl2::rect::Rect::new(
                pt.0 + (dst_offset * self.tile_dimension as f64) as i32,
                pt.1,
                size_px,
                self.tile_dimension);

            // canvas.set_draw_color(BLUE);
            // canvas.draw_rect(dst_rect);

            let dir = dst_node.dir_next;
            let angle = Self::get_direction_angle(&dir);

            // let dst_rect = rotate_rect(&dst_rect, &dir);

            canvas.set_draw_color(color);
            canvas.fill_rect(dst_rect);

            /*
            canvas.copy_ex(&self.bit,         // texture
                           src_rect,          // src rect
                           dst_rect,          // dst rect
                           angle, // angle of rotation
                           None,              // center for rotation -- None = dst (or src if dst None)
                           false,             // flip_horizontal
                           false);            // flip_vertical
                                              */
        }
    }

    /*
     * draw_snake_bit
     *
     * draw one unit of snake (1.0). this is one grid unit's worth of snake, but it can span upto
     * two (partial) grids. this is because it is lagged on the path by half_snake_width_normalized
     * and advanced by frame_percent. this is the usual case -- a given "bit" of snake is only
     * rendered within a single grid square when half_snake_width_normalized is equal to
     * frame_percent. in a given frame, all snake bits are rendered with the same frame_percent,
     * and each bit has the directional information for the previous and next bits, so it should
     * not matter what order the snake is iterated for rendering.
     *
     * */
    fn draw_snake_bit(&self,
                      frame_percent: f64,
                      game:          &GameState,
                      bit:           &CoordWithDirection, // to be drawn
                      prev:          &CoordWithDirection, // already drawn
                      next:          Option<&CoordWithDirection>, // drawn next
                      canvas:        &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        let start_percent = frame_percent - self.half_snake_width_normalized;
        let end_percent = start_percent + 1.0;

        if start_percent < 0.0 {

            // to previous grid: [0, -start_percent) -> [1.0 + start_percent, 1.0)
            self.draw_partial_snake_bit(
                bit,
                0.0,
                -start_percent,
                1.0 + start_percent,
                prev,
                game,
                canvas, BLACK);

            // to current grid: [-start_percent, 1.0) -> [0, end_percent)
            self.draw_partial_snake_bit(
                bit,
                -start_percent,
                end_percent,
                0.0,
                bit,
                game,
                canvas, RED);

        } else /* if start_percent >= 0.0 */ {
            // to current grid: [0, 1.0 - start_percent)  -> [start_percent, 1.0)
            self.draw_partial_snake_bit(
                bit,
                0.0,
                1.0 - start_percent,
                start_percent,
                bit,
                game,
                canvas, BLUE);

            let next_ = CoordWithDirection {
                dir_next: bit.dir_next,
                coord: bit.coord.calculate_neighbor(bit.dir_next),
                dir_prev: bit.dir_prev,
            };

            let next = match next {
                Some(node) => node,
                None => &next_,
            };

            // to previous grid: [1.0 - start_percent, 1.0) -> [0, start_percent)
            self.draw_partial_snake_bit(
                bit,
                1.0 - start_percent,
                start_percent,
                0.0,
                next,
                game,
                canvas, ORANGE);
        }

        let pt: (usize, usize) = game.game_to_grid_tuple(&bit.coord.as_tuple());

        let pt: (i32, i32) = (pt.0 as i32 * self.tile_dimension as i32,
                              pt.1 as i32 * self.tile_dimension as i32);

        let rect = sdl2::rect::Rect::new (
            pt.0 as i32 + self.tile_margin as i32,
            pt.1 as i32 + self.tile_margin as i32,
            self.snake_width as u32,
            self.snake_width as u32,
        );
        canvas.set_draw_color(RED);
        let _ = canvas.draw_rect(rect);
    }

    // just draw the center of the path for the grid
    fn draw_snake_path(&self,
                      frame_percent: f64,
                      game:          &GameState,
                      bit:           &CoordWithDirection, // to be drawn
                      prev:          &CoordWithDirection, // already drawn
                      next:          Option<&CoordWithDirection>, // drawn next
                      canvas:        &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        let pt: (usize, usize) = game.game_to_grid_tuple(&bit.coord.as_tuple());

        let c: (i32, i32) = (pt.0 as i32 * self.tile_dimension as i32,
                             pt.1 as i32 * self.tile_dimension as i32);

        // non-partial center of square
        let p: (i32, i32) = (c.0 + self.tile_dimension as i32 / 2,
                             c.1 + self.tile_dimension as i32 / 2);

        // canvas.filled_circle(pt.0 as i16, pt.1 as i16, 3, BLACK);

        // partial stuff stolen from head
        let target_direction = bit.dir_next;

        let incoming_direction = bit.dir_prev.get_opposite();

        let target_angle_deg: f64 = Self::get_direction_angle(&target_direction);
        let incoming_angle_deg: f64 = Self::get_direction_angle(&incoming_direction);


        // with respect to the center rather than the previous grid
        let incoming_angle_deg: f64 = Self::get_direction_angle(&incoming_direction.get_opposite());

        // incoming point
        canvas.filled_circle((p.0 as f64 + (self.half_snake_width_f64 * incoming_angle_deg.to_radians().cos())) as i16,
                             (p.1 as f64 + (self.half_snake_width_f64 * incoming_angle_deg.to_radians().sin())) as i16,
                             3,
                             RED);

        // exiting point
        canvas.filled_circle((p.0 as f64 + (self.half_snake_width_f64 * target_angle_deg.to_radians().cos())) as i16,
                             (p.1 as f64 + (self.half_snake_width_f64 * target_angle_deg.to_radians().sin())) as i16,
                             3,
                             SNAKE_COLOR_DARK);

        let average = (target_angle_deg - incoming_angle_deg) / 2.0;

        let direction_kind = match (incoming_direction, target_direction) {
            (Direction::Up, Direction::Right) => DirectionKind::Anticlockwise,
            (Direction::Up, Direction::Left) => DirectionKind::Clockwise,
            (Direction::Down, Direction::Left) => DirectionKind::Anticlockwise,
            (Direction::Down, Direction::Right) => DirectionKind::Clockwise,
            (Direction::Left, Direction::Up) => DirectionKind::Anticlockwise,
            (Direction::Left, Direction::Down) => DirectionKind::Clockwise,
            (Direction::Right, Direction::Down) => DirectionKind::Anticlockwise,
            (Direction::Right, Direction::Right) => DirectionKind::Clockwise,
            _ => DirectionKind::Straight,
        };

        // is this a straight line or angle?
        let diff_deg = (target_angle_deg - incoming_angle_deg).abs();
        if diff_deg == 180.0 {
            // straight line
        } else {
            let mut angle_to_corner = ((incoming_angle_deg - target_angle_deg) / 2.0) + target_angle_deg;
            if incoming_angle_deg < target_angle_deg {
                angle_to_corner = angle_to_corner + 180.0;
            }
            if direction_kind == DirectionKind::Clockwise {
                angle_to_corner = angle_to_corner + 180.0;
            }
            if angle_to_corner > 360.0 {
                angle_to_corner = angle_to_corner - 360.0;
            }
            let corner = (p.0 as f64 + (self.tile_dimension as f64 / 2.0 * angle_to_corner.to_radians().cos()),
                          p.1 as f64 + (self.tile_dimension as f64 / 2.0 * angle_to_corner.to_radians().sin()));
            // canvas.draw_line(sdl2::rect::Point::new(corner.0 as i32, corner.1 as i32), sdl2::rect::Point::new(p.0 as i32, p.1 as i32));
            canvas.filled_circle(corner.0 as i16, corner.1 as i16, 3, ORANGE);
            // canvas.arc(corner.0 as i16, corner.1 as i16, self.tile_dimension as i16 / 2, angle_to_corner as i16 - 45, angle_to_corner as i16 + 45, BLUE);
        }

        // angle of forward direction
        let forward_angle_deg: f64 = match (incoming_direction, target_direction) {
            // rotate special cases in correct direction
            (Direction::Up, Direction::Right)
                => incoming_angle_deg + ((360.0 - incoming_angle_deg) * frame_percent),

            (Direction::Right, Direction::Up)
                => 360.0 + ((target_angle_deg - 360.0) * frame_percent),

            // general case rotates in correct direction
            _   => incoming_angle_deg + ((target_angle_deg - incoming_angle_deg) * frame_percent),
        };

        println!("In={:?}:{}; Out={:?}:{}; fwd={:?}; avg={:?}", incoming_direction, incoming_angle_deg, target_direction, target_angle_deg, forward_angle_deg, average);

        // find target root point in grid
        let (tx, ty) = (frame_percent * self.half_snake_width_f64 * forward_angle_deg.to_radians().cos(),
                        frame_percent * self.half_snake_width_f64 * forward_angle_deg.to_radians().sin());

        // translate rotated surface root point to grid
        let (sx, sy) = (p.0 + tx as i32, p.1 + ty as i32);

        // canvas.filled_circle(sx as i16, sy as i16, 3, BLUE);

        // draw incoming point
        // let (tx, ty) = (self.half_snake_width_f64 * incoming_angle_deg.to_radians().cos(),
        //                 self.half_snake_width_f64 * incoming_angle_deg.to_radians().sin());

    }

    fn draw_snake_bit_simple(&self,
                             frame_percent: f64,
                             game:          &GameState,
                             bit:           &CoordWithDirection, // to be drawn
                             prev:          &CoordWithDirection, // already drawn
                             next:          Option<&CoordWithDirection>, // drawn next
                             canvas:        &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        let pt: (usize, usize) = game.game_to_grid_tuple(&bit.coord.as_tuple());

        let pt: (i32, i32) = (pt.0 as i32 * self.tile_dimension as i32,
                              pt.1 as i32 * self.tile_dimension as i32);

        let rect = sdl2::rect::Rect::new (
            pt.0 as i32 + self.tile_margin as i32,
            pt.1 as i32 + self.tile_margin as i32,
            self.snake_width as u32,
            self.snake_width as u32,
        );

        canvas.set_draw_color(RED);
        let _ = canvas.draw_rect(rect);
    }

    pub fn draw_snake(&mut self,
                      frame_percent: f64,
                      game: &GameState,
                      canvas: &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        canvas.set_draw_color(SNAKE_COLOR_LIGHT);

        let mut iter = game.get_snake().get_body().iter();

        if let Some(head) = iter.next() {
            let mut prev = head;
            let next_bit = iter.next();

            println!("frame_percent: {}", frame_percent);

            self.draw_animated_snake_head(frame_percent, game, head, next_bit, canvas);
            // self.draw_snake_bit(frame_percent, game, head, bit, None, canvas);
            match next_bit {
                Some(bit) => {

                    self.draw_snake_path(frame_percent, game, head, bit, None, canvas);
                    self.draw_snake_bit_simple(frame_percent, game, head, bit, None, canvas);

                    let mut cur = bit;
                    for following in iter {
                        // middle
                        // self.draw_snake_bit(frame_percent, game, cur, prev, Some(&following), canvas);
                        self.draw_snake_bit_simple(frame_percent, game, cur, prev, Some(&following), canvas);
                        self.draw_snake_path(frame_percent, game, cur, prev, Some(&following), canvas);
                        prev = cur;
                        cur = following;
                    }
                    // tail
                    // self.draw_snake_bit(frame_percent, game, cur, prev, None, canvas);
                    self.draw_snake_bit_simple(frame_percent, game, cur, prev, None, canvas);
                    self.draw_snake_path(frame_percent, game, cur, prev, None, canvas);
                },
                None => (),
            }
        } else {
            panic!("headless snake!");
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

        let incoming_direction = at.dir_prev.get_opposite();

        let target_angle_deg: f64 = Self::get_direction_angle(&target_direction);
        let incoming_angle_deg: f64 = Self::get_direction_angle(&incoming_direction);

        // angle of forward direction
        let forward_angle_deg: f64 = match (incoming_direction, target_direction) {
            // rotate special cases in correct direction
            (Direction::Up, Direction::Right)
                => incoming_angle_deg + ((360.0 - incoming_angle_deg) * frame_percent),

            (Direction::Right, Direction::Up)
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
}

fn rotate_rect(rect: &sdl2::rect::Rect,
               direction: &Direction)
    -> sdl2::rect::Rect
{
    match direction {
        Direction::Right | Direction::Left => rect.clone(),
        Direction::Down | Direction::Up => sdl2::rect::Rect::new(
            rect.x + rect.w - rect.h,
            rect.y + rect.h - rect.w,
            rect.h as u32,
            rect.w as u32,
        ),
    }
}

fn rotate_rect_old(center: &(i32, i32), rect: &sdl2::rect::Rect, direction: &Direction) -> sdl2::rect::Rect {

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
