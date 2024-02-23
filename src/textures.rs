// provides pie and filled_pie for sdl2::render::Canvas
use sdl2::gfx::primitives::DrawRenderer;

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

    let window = video_subsystem
        .window("textures.rs - SDL2 Driver", WIDTH_PIXELS, HEIGHT_PIXELS)
        .position(0, 0)
        .build()
        .unwrap();

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

    ctx.draw();

    while ctx.get_input() {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}

fn reverse_transform(bitmap_index: usize, d: usize, bpp: usize) -> Option<usize> {

    let channel = bitmap_index % bpp;

    let pixel_index = (bitmap_index - channel) / bpp;

    let x_px = pixel_index % d;
    let y_px = (pixel_index - x_px) / d;

    // normalize
    let x = x_px as f64 / d as f64;
    let y = y_px as f64 / d as f64;

    // reverse
    let x2 = x.powi(2)  + y.powi(2);
    if x2 > 1.0 {
        return None;
    }

    let xr = x2.sqrt();
    // x and y are all [0,1] so atan2 is always in the first quadrant
    let yr = y.atan2(x) / std::f64::consts::FRAC_PI_2;

    // scale back to pixels
    let xb = (xr * d as f64) as usize;
    let yb = (yr * d as f64) as usize;

    // and index
    let i1 = (((yb * d) + xb) * bpp) + channel;

    return Some(i1);
}

fn forward_transform(bitmap_index: usize, d: usize, bpp: usize) -> usize {

    let channel = bitmap_index % bpp;

    let pixel_index = (bitmap_index - channel) / bpp;

    let x_px = pixel_index % d;
    let y_px = (pixel_index - x_px) / d;

    // normalize
    let x = x_px as f64 / d as f64;
    let y = y_px as f64 / d as f64;

    // reverse
    // let xr = x.powi(2) + y.powi(2).sqrt();
    // let yr = y.atan2(x) / std::f64::consts::FRAC_PI_2;
    let xr = x * (y * std::f64::consts::FRAC_PI_2).cos();
    let yr = x * (y * std::f64::consts::FRAC_PI_2).sin();

    // scale back to pixels
    let xb = (xr * d as f64) as usize;
    let yb = (yr * d as f64) as usize;

    // and index
    let i1 = (((yb * d) + xb) * bpp) + channel;

    return i1;
}

impl SDLContext<'_> {

    fn draw_circle(&mut self, cx: i32, cy: i32, or: i32, ir: i32) {
        // draw a box
        let texture_creator = self.canvas.texture_creator();

        let d = (ir * 2) + or;

        let mut texture = texture_creator.create_texture(None, sdl2::render::TextureAccess::Target, d as u32, d as u32).unwrap();

        let textures = vec![(&mut texture, ()),];
        let result = self.canvas.with_multiple_texture_canvas(&mut textures.iter(), |tc, _| {
            tc.filled_pie(0, 0, or as i16, 0, 90, GREEN);
            tc.pie(0, 0, (ir + (or / 2)) as i16, 0, 90, BLACK);
            tc.filled_pie(0, 0, ir as i16, 0, 90, BLACK);
        });
        self.canvas.copy(&texture, None, sdl2::rect::Rect::new(cx - or - ir, cy - or - ir, d as u32, d as u32));
    }

    fn pixel_test (&mut self, cx: i32, cy: i32, d: i32) {

        let texture_creator = self.canvas.texture_creator();

        let fmt = sdl2::pixels::PixelFormatEnum::RGB24;
        let bpp = 3;

        let mut ssrc = sdl2::surface::Surface::new(d as u32, d as u32, fmt).unwrap();
        let mut sdst = sdl2::surface::Surface::new(d as u32, d as u32, fmt).unwrap();
        let mut sdst2 = sdl2::surface::Surface::new(d as u32, d as u32, fmt).unwrap();
        let mut sdst3 = sdl2::surface::Surface::new(d as u32, d as u32, fmt).unwrap();

        // create random data
        for i in 0..10 {
            let cx = rand::thread_rng().gen_range(0..d) as i16;
            let cy = rand::thread_rng().gen_range(0..d) as i16;
            let r = rand::thread_rng().gen_range(0..(d / 2)) as i16;
            let t0 = rand::thread_rng().gen_range(0..270) as i16;
            let t1 = rand::thread_rng().gen_range((t0 + 1)..360) as i16;
            ssrc.fill_rect(sdl2::rect::Rect::new(cx as i32, cy as i32, (cx + r) as u32, (cy + r) as u32), sdl2::pixels::Color::RGB(rand::thread_rng().gen_range(0..=255),rand::thread_rng().gen_range(0..=255),rand::thread_rng().gen_range(0..=255)));
        }
        ssrc.fill_rect(sdl2::rect::Rect::new(0, 0, 50, 50), sdl2::pixels::Color::RGB(0,127,255));

        // transform it
        ssrc.with_lock(|src| {
            sdst.with_lock_mut(|dst| {
                for (i, e) in src.iter().enumerate() {
                    dst[i] = *e;
                }
            });
        });

        ssrc.with_lock(|src| {
            sdst2.with_lock_mut(|dst| {
                for (i, e) in src.iter().enumerate() {
                    let i1 = forward_transform(i, d as usize, bpp as usize);
                    if i1 >= 0 && i1 < (d * d * bpp) as usize {
                        dst[i1] = *e;
                    }
                }
            });
        });

        ssrc.with_lock(|src| {
            sdst3.with_lock_mut(|dst| {
                for i in 0..((d*d*bpp) as usize) {
                    if let Some(i1) = reverse_transform(i, d as usize, bpp as usize) {
                        if i1 >= 0 && i1 < (d * d * bpp) as usize {
                            dst[i] = src[i1];
                        }
                    }
                }
            });
        });

        let mut tsrc = ssrc.as_texture(&texture_creator).unwrap();
        let mut tdst = sdst.as_texture(&texture_creator).unwrap();
        let mut tdst2 = sdst2.as_texture(&texture_creator).unwrap();
        let mut tdst3 = sdst3.as_texture(&texture_creator).unwrap();

        self.canvas.copy(&tsrc, None, sdl2::rect::Rect::new(cx - (d / 2), cy - (d / 2), d as u32, d as u32));
        self.canvas.copy(&tdst, None, sdl2::rect::Rect::new(cx - (d / 2) + d + 20, cy - (d / 2), d as u32, d as u32));
        self.canvas.copy(&tdst2, None, sdl2::rect::Rect::new(cx - (d / 2) + (d * 2) + 40, cy - (d / 2), d as u32, d as u32));
        self.canvas.copy(&tdst3, None, sdl2::rect::Rect::new(cx - (d / 2) + (d * 3) + 60, cy - (d / 2), d as u32, d as u32));
    }

    fn draw(&mut self) {
        // self.canvas.set_draw_color(sdl2::pixels::Color::RGB(self.color_index, 64, 255 - self.color_index));
        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        self.canvas.clear();

        self.canvas.set_draw_color(RED);

        self.pixel_test(100, 300, 200 as i32);

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
