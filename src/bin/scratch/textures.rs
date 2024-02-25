// provides pie and filled_pie for sdl2::render::Canvas
// bezier and filled_pie (maybe other primitives) did not appear when drawn on a RGB888 surface,
// showed up on RGBA8888
use sdl2::gfx::primitives::DrawRenderer;

use sdl2::gfx::rotozoom::RotozoomSurface;

use rand::Rng;

const WIDTH_PIXELS: u32 = 1200;
const HEIGHT_PIXELS: u32 = 750;

const G2PX: u32 = 50;

const CELL_MARGIN_PX: u32 = 4;

const WIDTH: u32 = WIDTH_PIXELS / G2PX;
const HEIGHT: u32 = HEIGHT_PIXELS / G2PX;

const FOOD_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(200, 200, 20);
const SNAKE_COLOR: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 200, 50);
const SNAKE_COLOR_DARK: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 150, 60);
const RED: sdl2::pixels::Color = sdl2::pixels::Color::RGB(255, 0, 0);
const BLUE: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 255);
const GREEN: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 255, 0);
const BLACK: sdl2::pixels::Color = sdl2::pixels::Color::RGB(0, 0, 0);

struct SDLContext<'a> {
    event_pump: &'a mut sdl2::EventPump,
    canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>,
}

fn main() {

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let mut window = video_subsystem
        .window("textures.rs - SDL2 Driver", WIDTH_PIXELS, HEIGHT_PIXELS)
        .position(0, 0)
        .build()
        .unwrap();

    let mut display_mode = window.display_mode().unwrap();
    println!("display_mode: {:?}", display_mode);

    display_mode.format = sdl2::pixels::PixelFormatEnum::RGBA8888;

    match window.set_display_mode(display_mode) {
        Ok(_) => println!("set_display_mode OK!"),
        Err(s) => println!("error setting display mode '{}'", s),
    }

    let mut canvas = Some(window.into_canvas().build().unwrap());

    if let Some(ref mut canvas_here) = canvas {
        canvas_here.set_draw_color(sdl2::pixels::Color::RGB(0, 255, 255));
        canvas_here.clear();
        canvas_here.present();
    }

    let mut ctx: SDLContext = SDLContext {
        canvas: &mut canvas.unwrap(),
        event_pump: &mut sdl_context.event_pump().unwrap(),
    };

    ctx.bezier_test();

    while ctx.get_input() {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}

fn index_to_xyc(index: usize, width: usize, bpp: usize) -> (usize, usize, usize) {

    let channel: usize = index % bpp;
    let pixel_index: usize = (index - channel) / bpp;

    let x_px: usize = pixel_index % width;
    let y_px: usize = (pixel_index - x_px) / width;

    // println!("index_to_xyc: {}@({},{}) -> ({},{},{})", index, width, bpp, x_px, y_px, channel);

    (x_px, y_px, channel)
}

fn xyc_to_index(x: usize, y: usize, channel: usize, width: usize, bpp: usize) -> usize {
    let out = (((y * width) + x) * bpp) + channel;
    // println!("xyc_to_index: ({},{},{})@({},{}) -> {}", x,y,channel, width, bpp, out);
    return out;
}

// round-trip it
fn test_transform_roundtrip(x: usize, y: usize, channel: usize, width: usize, bpp: usize) {
    let i0 = xyc_to_index(x,y,channel,width,bpp);
    let i1 = reverse_transform(i0,width,bpp, 0.0, 1.0).unwrap();
    let (x1,y1,c1) = index_to_xyc(i1, width, bpp);

    // println!("({},{},{}) vs ({},{},{})", x,y,channel, x1,y1,c1);
}

fn transform_test(width: usize, x: usize, y: usize) {
    let bpp = 3;
    let c = 0;
    let i = xyc_to_index(x,y,c,width,bpp);
    if let Some(out) = reverse_transform(i, width, bpp, 0.0, 1.0) {
        let (x1, y1, c1) = index_to_xyc(out, width, bpp);
        // println!("{}: ({},{},{})={} -> {}=({},{},{})", width, x,y,c,i, out,x1,y1,c1);
    } else {
        // println!("{}: {} -> None!", width, i);
    };
}

fn reverse_transform(bitmap_index: usize, width: usize, bpp: usize, inner_radius: f64, outer_radius: f64) -> Option<usize> {

    let (x_px, y_px, channel) = index_to_xyc(bitmap_index, width, bpp);

    // normalize
    let x: f64 = x_px as f64; // / width as f64;
    let y: f64 = y_px as f64; // / width as f64;

    //
    // reverse
    //

    // radius check
    let x2: f64 = x.powi(2)  + y.powi(2);
    if x2 < (inner_radius * width as f64).powi(2) || (outer_radius * width as f64).powi(2) < x2 {
        return None;
    }
    let xr: f64 = x2.sqrt();

    let yr: f64 = width as f64 * y.atan2(x) / std::f64::consts::FRAC_PI_2;

    // scale back to pixels
    let xb: usize = (xr /* * width as f64*/) as usize;
    let yb: usize = (yr /* * width as f64*/) as usize;

    let out = xyc_to_index(xb, yb, channel, width, bpp);

    let (xo, yo, co) = index_to_xyc(out, width, bpp);

    if x_px == 32 && y_px == 30 && channel == 0 {
        println!("({},{}) @ ({},{}) -> ({},{}) -> ({},{}) -> ({},{}) -> {} -> ({},{},{})", width, channel, x_px, y_px, x, y, xr, yr, xb, yb, out, xo,yo,co);
    }

    // and index
    return Some(out);
}

fn forward_transform(bitmap_index: usize, d: usize, bpp: usize) -> usize {

    let (x_px, y_px, channel) = index_to_xyc(bitmap_index, d, bpp);

    let channel = bitmap_index % bpp;

    let pixel_index = (bitmap_index - channel) / bpp;

    let x_px = pixel_index % d;
    let y_px = (pixel_index - x_px) / d;

    // normalize
    let x = x_px as f64; //  / d as f64;
    let y = y_px as f64; //  / d as f64;

    // forward
    let xr = x * ((y / d as f64) * std::f64::consts::FRAC_PI_2).cos();
    let yr = x * ((y / d as f64) * std::f64::consts::FRAC_PI_2).sin();

    // scale back to pixels
    let xb = (xr /* * d as f64*/) as usize;
    let yb = (yr /* * d as f64*/) as usize;

    // and index
    return xyc_to_index(xb, yb, channel, d, bpp);
}

fn xy_reflect(bitmap_index: usize, width: usize, bpp: usize) -> usize {
    let (x_px, y_px, channel) = index_to_xyc(bitmap_index, width, bpp);
    return xyc_to_index(y_px, x_px, channel, width, bpp);
}

fn simple_rotate(bitmap_index: usize, width: usize, bpp: usize) -> usize {
    let (x_px, y_px, channel) = index_to_xyc(bitmap_index, width, bpp);

    return xyc_to_index(width - x_px - 1, y_px , channel, width, bpp);
}

fn identity(bitmap_index: usize, width: usize, bpp: usize) -> usize {
    let (x_px, y_px, channel) = index_to_xyc(bitmap_index, width, bpp);
    return xyc_to_index(x_px, y_px, channel, width, bpp);
}

fn fill_surface_random(d: i32, fmt: sdl2::pixels::PixelFormatEnum) -> sdl2::surface::Surface<'static> {
    let mut s = sdl2::surface::Surface::new(d as u32, d as u32, fmt).unwrap();

    // create random data
    for i in 0..10 {
        let cx = rand::thread_rng().gen_range(0..d) as i16;
        let cy = rand::thread_rng().gen_range(0..d) as i16;
        let r = rand::thread_rng().gen_range(0..(d / 2)) as i16;
        let t0 = rand::thread_rng().gen_range(0..270) as i16;
        let t1 = rand::thread_rng().gen_range((t0 + 1)..360) as i16;
        s.fill_rect(sdl2::rect::Rect::new(cx as i32, cy as i32, (cx + r) as u32, (cy + r) as u32), sdl2::pixels::Color::RGB(rand::thread_rng().gen_range(0..=255),rand::thread_rng().gen_range(0..=255),rand::thread_rng().gen_range(0..=255)));
    }

    s.fill_rect(sdl2::rect::Rect::new(0, 0, 10, 10), sdl2::pixels::Color::RGB(0,127,255));

    return s;
}

fn fill_surface_bezier(d: i32, fmt: sdl2::pixels::PixelFormatEnum) -> sdl2::surface::Surface<'static> {
    let mut s = sdl2::surface::Surface::new(d as u32, d as u32, fmt).unwrap();

    let mut canvas = s.into_canvas().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
    canvas.clear();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 255, 0));

    // let mut rng = rand::thread_rng();

    // anywhere from 3 to 7 pts
    // normally distributed about the "edge"
    // y component is beta-1/2-distributed between 0 and end

    let n: usize = rand::thread_rng().gen_range(3..=7);

    let razor_margin = (0.05 * d as f64) as i16;
    let snake_edge_width = (0.15 * d as f64) as i16;

    let gen_xs = rand::distributions::Uniform::new(razor_margin, snake_edge_width);
    let gen_ys = rand::distributions::Uniform::new(razor_margin, d as i16 - razor_margin);

    let xs: Vec<i16> = rand::thread_rng()
        .sample_iter(&gen_xs)
        .take(n)
        .collect();

    let mut ys: Vec<i16> = rand::thread_rng()
        .sample_iter(gen_ys)
        .take(n - 2)
        .collect::<Vec<i16>>();

    ys.push(0);
    ys.push((d - 1) as i16);

    ys.sort();

    println!("{:?}, {:?}", xs, ys);

    // snake body is 2 beziers with control points centered around lines 20% in from the edge
    let x = canvas.bezier(&xs, &ys, n as i32, GREEN);

    // canvas.present();

    let mut s2 = canvas.into_surface();

    // s2.fill_rect(sdl2::rect::Rect::new(0, 0, 10, 10), sdl2::pixels::Color::RGB(0,127,255));

    return s2;
}

impl SDLContext<'_> {

    fn pixel_test (&mut self, cx: i32, cy: i32, ssrc: &sdl2::surface::Surface, bpp: usize) {

        let texture_creator = self.canvas.texture_creator();

        let h = ssrc.height();
        let w = ssrc.width();
        let fmt = ssrc.pixel_format_enum();

        println!("({},{})@{:?}", w,h,fmt);

        let pixel_buffer_size: usize = (w * h) as usize * bpp;

        let mut dsts: Vec<sdl2::surface::Surface> = vec![];

        for i in 0..4 {
            dsts.push(sdl2::surface::Surface::new(w, h, fmt).unwrap());
        }

        // copy
        ssrc.with_lock(|src| {
            dsts[0].with_lock_mut(|dst| {
                for (i, e) in src.iter().enumerate() {
                    dst[i] = *e;
                }
            });
        });

        /*
        // copy, then rotate-self
        ssrc.blit(None, &mut dsts[1], None);
        dsts[1].rotate_90deg(1);

        // simple rotate
        ssrc.with_lock(|src| {
            dsts[2].with_lock_mut(|dst| {
                for (i, e) in src.iter().enumerate() {
                    let i1 = xy_reflect(i, w as usize, bpp as usize);
                    dst[i1] = *e;
                }
            });
        });
        */

        // forward transform
        ssrc.with_lock(|src| {
            dsts[1].with_lock_mut(|dst| {
                for (i, e) in src.iter().enumerate() {
                    let i1 = forward_transform(i, w as usize, bpp as usize);
                    if i1 >= 0 && i1 < pixel_buffer_size {
                        dst[i1] = *e;
                    }
                }
            });
        });

        // reverse transform
        ssrc.with_lock(|src| {
            dsts[2].with_lock_mut(|dst| {
                for i in 0..pixel_buffer_size {
                    if let Some(i1) = reverse_transform(i, w as usize, bpp as usize, 0.0, 1.0) {
                        if i1 >= 0 && i1 < pixel_buffer_size {
                            dst[i] = src[i1];
                        }
                    }
                }
            });
        });

        // rotozoom then copy
        let rzed = ssrc.rotozoom(30_f64,
                                 1.0,
                                 false).unwrap();
        println!("ssrc: ({},{})", ssrc.width(), ssrc.height());
        println!("rzed: ({},{})", rzed.width(), rzed.height());
        rzed.blit(sdl2::rect::Rect::new(0, 0, w, h), &mut dsts[3], None);

        /*
        // identity transform
        ssrc.with_lock(|src| {
            dsts[5].with_lock_mut(|dst| {
                for i in 0..pixel_buffer_size {
                    let i1 = identity(i, w as usize, bpp as usize);
                    dst[i] = src[i1];
                }
            });
        });

        // identity transform
        ssrc.with_lock(|src| {
            dsts[6].with_lock_mut(|dst| {
                for i in 0..pixel_buffer_size {
                    let i1 = simple_rotate(i, w as usize, bpp as usize);
                    dst[i] = src[i1];
                }
            });
        });

        // hooray beer
        dsts[7].with_lock_mut(|dst| {
            for i in 0..pixel_buffer_size {
                let (x,y,c) = index_to_xyc(i, w as usize, bpp as usize);

                if c == 0 {
                    if x % 20 >= 10 {
                        dst[i] = 0xff;
                    }
                }
            }
        });
        */

        let mut texs: Vec<sdl2::render::Texture> = vec![];

        texs.push(ssrc.as_texture(&texture_creator).unwrap());
        for dst in dsts.iter() {
            texs.push(dst.as_texture(&texture_creator).unwrap());
        }

        for (i, tex) in texs.iter().enumerate() {
            self.canvas.copy(&tex, None, sdl2::rect::Rect::new(cx + (i as u32 * (w + 20)) as i32, cy, w as u32, h as u32));
        }

        for i in 0..12 {
            let tx = cx + (i as u32 * (w + 20)) as i32;
            let ty = cy + h as i32 + 50;
            let a = i as f64 * (360_f64 / 12_f64);
            self.canvas.copy_ex(&texs[0],
                                None,
                                sdl2::rect::Rect::new(tx, ty, w, h),
                                a,
                                None,
                                false,
                                false);

            self.canvas.circle(tx as i16, ty as i16, 3, RED);

            self.canvas.filled_circle((tx + (w / 2) as i32) as i16, (ty + (h / 2) as i32) as i16, 3, BLUE);

            // try to draw the rotated bounding box to prove we know how to find it
            let bbrect = self.calculate_rotated_bounding_box_rect(tx as f64 + (w / 2) as f64,
                                                                  ty as f64 + (h / 2) as f64,
                                                                  w as f64 / 2.0,
                                                                  a);

            self.canvas.set_draw_color(BLUE);
            self.canvas.draw_rect(bbrect);
        }
    }

    fn calculate_rotated_bounding_box_rect(&mut self,
                                           x0: f64,
                                           y0: f64,
                                           r: f64,
                                           angle: f64)
        -> sdl2::rect::Rect
    {

        let angle = angle * std::f64::consts::PI / 180.0;
        let (cos, sin) = (angle.cos(), angle.sin());
        let (rc, rs) = (r * cos, r * sin);

        // points in each quadrant
        let q1 = ( rc - rs,  rs + rc);
        let q2 = (-rc - rs, -rs + rc);
        let q3 = (-rc + rs, -rs - rc);
        let q4 = ( rc + rs,  rs - rc);

        self.canvas.filled_circle((x0 + q1.0) as i16, (y0 + q1.1) as i16, 3, RED);
        self.canvas.filled_circle((x0 + q2.0) as i16, (y0 + q2.1) as i16, 3, BLUE);
        self.canvas.filled_circle((x0 + q3.0) as i16, (y0 + q3.1) as i16, 3, SNAKE_COLOR_DARK);
        self.canvas.filled_circle((x0 + q4.0) as i16, (y0 + q4.1) as i16, 3, BLACK);

        let mut pts = vec![q1,q2,q3,q4];
        let x = *pts.iter().map(|p| p.0 as i32).collect::<Vec<_>>().iter().min().unwrap();
        let y = *pts.iter().map(|p| p.1 as i32).collect::<Vec<_>>().iter().min().unwrap();
        let w = *pts.iter().map(|p| p.0 as i32).collect::<Vec<_>>().iter().max().unwrap() - x;
        let h = *pts.iter().map(|p| p.1 as i32).collect::<Vec<_>>().iter().max().unwrap() - y;

        return sdl2::rect::Rect::new(x0 as i32 + x, y0 as i32 + y, w as u32, h as u32);
    }

    fn small_big_compare(&mut self) {
        // self.canvas.set_draw_color(sdl2::pixels::Color::RGB(self.color_index, 64, 255 - self.color_index));
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();

        self.canvas.set_draw_color(RED);

        let big = 60;
        let small = 63;

        let fmt = sdl2::pixels::PixelFormatEnum::RGB24;

        let s60: sdl2::surface::Surface = fill_surface_random(big, fmt);

        let mut s59 = sdl2::surface::Surface::new(small, small, fmt).unwrap();

        println!("big: {}, small: {}", s60.pitch(), s59.pitch());

        let rect = sdl2::rect::Rect::new(0,0,small, small);

        s60.blit(rect, &mut s59, None);

        self.pixel_test(100, 100, &s60, 3);
        self.pixel_test(100, 150 + big, &s59, 3);

        for i in 0..59 {
            transform_test(60, i, i);
            transform_test(59, i, i);
        }

        test_transform_roundtrip(30,49,0,60,3);
        test_transform_roundtrip(30,49,0,59,3);

        self.canvas.present();
    }

    fn bezier_test(&mut self) {
        // self.canvas.set_draw_color(sdl2::pixels::Color::RGB(self.color_index, 64, 255 - self.color_index));
        // self.canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();

        let fmt = sdl2::pixels::PixelFormatEnum::RGBA8888;

        let bezier_surface: sdl2::surface::Surface = fill_surface_bezier(100, fmt);

        self.pixel_test(100, 100, &bezier_surface, 4);

        for i in 0..59 {
            transform_test(60, i, i);
            transform_test(59, i, i);
        }

        test_transform_roundtrip(30,49,0,60,3);
        test_transform_roundtrip(30,49,0,59,3);

        let x = self.canvas.bezier(&[40, 80, 30, 40], &[0, 50, 150, 199], 3, BLACK);

        self.canvas.present();
    }

    fn get_input(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    return false;
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Q),
                    ..
                } => {
                    return false;
                }
                _ => true,
            };
        }
        return true;
    }
}

