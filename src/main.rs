extern crate failure;
extern crate floating_duration;
extern crate sdl2;

use std::time::Instant;

use failure::{err_msg, Error};
use floating_duration::TimeAsFloat;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::FullscreenType;

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

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        let tile_size = 16;
        canvas.set_logical_size(self.width * tile_size, self.height * tile_size)?;
        canvas.set_draw_color(Color::RGB(0x20, 0x20, 0x20));
        canvas.clear();
        for (i, tile) in self.tiles.iter().enumerate() {
            let x = i as i32 % self.width as i32 * tile_size as i32;
            let y = i as i32 / self.width as i32 * tile_size as i32;
            let tile_color = match *tile {
                Tile::Empty => Color::RGB(0x00, 0x00, 0x00),
                Tile::Filled => Color::RGB(0x80, 0x80, 0x80),
            };
            canvas.set_draw_color(tile_color);
            canvas
                .fill_rect(Rect::new(x, y, tile_size, tile_size))
                .map_err(err_msg)?;
        }
        Ok(())
    }
}

pub struct Player {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            x: 10.0,
            y: 10.0,
            dx: 0.0,
            dy: 0.0,
        }
    }

    pub fn left_pressed(&mut self) {
        self.dx = -60.0;
    }

    pub fn left_released(&mut self) {
        if self.dx < 0.0 {
            self.dx = 0.0;
        }
    }

    pub fn right_pressed(&mut self) {
        self.dx = 60.0;
    }

    pub fn right_released(&mut self) {
        if self.dx > 0.0 {
            self.dx = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.dx * dt;
        self.y += self.dy * dt;
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        let x = self.x.round() as i32;
        let y = self.y.round() as i32;
        canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        canvas.fill_rect(Rect::new(x, y, 8, 20)).map_err(err_msg)?;
        Ok(())
    }
}

fn run() -> Result<(), Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video = sdl.video().map_err(err_msg)?;
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    let window = video.window("Grot", 640, 480).build()?;
    let mut canvas = window.into_canvas().present_vsync().build()?;

    let mut last_update_time = Instant::now();
    let room = Room::new(20, 10);
    let mut player = Player::new();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(()),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    repeat: false,
                    ..
                } => {
                    // Toggle fullscreen state
                    let window = canvas.window_mut();
                    let fullscreen_state = window.fullscreen_state();
                    window
                        .set_fullscreen(match fullscreen_state {
                            FullscreenType::Off => FullscreenType::Desktop,
                            _ => FullscreenType::Off,
                        })
                        .map_err(err_msg)?;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                } => player.left_pressed(),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => player.left_released(),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                } => player.right_pressed(),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => player.right_released(),
                _ => eprintln!("Unhandled event of type {:?}", event),
            }
        }

        // Use a simple variable timestep for updating the game state for now.
        // See also https://gafferongames.com/post/fix_your_timestep/
        let now = Instant::now();
        let time_delta = (now - last_update_time).as_fractional_secs() as f32;
        eprintln!("Time delta: {}", time_delta);
        player.update(time_delta);
        last_update_time = now;
        room.render(&mut canvas)?;
        player.render(&mut canvas)?;
        canvas.present();
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
