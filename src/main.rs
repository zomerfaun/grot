extern crate failure;
extern crate sdl2;

use failure::{err_msg, Error};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;

#[derive(Clone, Copy)]
pub enum Tile {
    Empty,
    Filled,
}

pub struct Room {
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl Room {
    pub fn new(width: u32, height: u32) -> Room {
        // Construct an empty roomsworth of tiles
        let mut tiles = vec![Tile::Empty; (width * height) as usize];

        // Add a floor
        for x in 0..width as usize {
            tiles[(width * (height - 1)) as usize + x] = Tile::Filled;
        }

        Room {
            width,
            height,
            tiles,
        }
    }

    pub fn render<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
    ) -> Result<(), Error> {
        let tile_size = 16;
        canvas.set_logical_size(self.width * tile_size, self.height * tile_size)?;
        canvas.set_draw_color(Color::RGB(0x20, 0x20, 0x20));
        canvas.clear();
        for (i, tile) in self.tiles.iter().enumerate() {
            let x = i as i32 % self.width as i32 * tile_size as i32;
            let y = i as i32 / self.width as i32 * tile_size as i32;
            let draw_color = match *tile {
                Tile::Empty => Color::RGB(0, 0, 0),
                Tile::Filled => Color::RGB(255, 255, 255),
            };
            canvas.set_draw_color(draw_color);
            canvas
                .fill_rect(Rect::new(x, y, tile_size, tile_size))
                .map_err(err_msg)?;
        }
        canvas.present();
        Ok(())
    }
}

fn run() -> Result<(), Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video = sdl.video().map_err(err_msg)?;
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    let window = video.window("Grot", 640, 480).build()?;
    let mut canvas = window.into_canvas().present_vsync().build()?;

    let room = Room::new(20, 10);

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                _ => (),
            }
        }
        room.render(&mut canvas)?;
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        for cause in error.causes().skip(1) {
            eprintln!("Cause: {}", cause);
        }
        eprintln!("{}", error.backtrace());
        std::process::exit(1);
    }
}
