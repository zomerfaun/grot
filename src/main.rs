extern crate failure;
extern crate floating_duration;
extern crate sdl2;

use std::ops::{AddAssign, Mul};
use std::thread;
use std::time::{Duration, Instant};

use failure::{err_msg, Error};
use floating_duration::{TimeAsFloat, TimeFormat};
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

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl Default for Vec2 {
    fn default() -> Vec2 {
        Vec2::new(0.0, 0.0)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

pub struct Player {
    position: Vec2,
    speed: Vec2,
    acceleration: Vec2,
}

impl Player {
    pub fn new() -> Player {
        Player {
            position: Vec2::new(10.0, 10.0),
            speed: Vec2::default(),
            acceleration: Vec2::default(),
        }
    }

    pub fn left_pressed(&mut self) {
        self.speed.x = -60.0;
    }

    pub fn left_released(&mut self) {
        if self.speed.x < 0.0 {
            self.speed.x = 0.0;
        }
    }

    pub fn right_pressed(&mut self) {
        self.speed.x = 60.0;
    }

    pub fn right_released(&mut self) {
        if self.speed.x > 0.0 {
            self.speed.x = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Semi-implicit Euler integration
        // See https://gafferongames.com/post/integration_basics/
        self.speed += self.acceleration * dt;
        self.position += self.speed * dt;
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        let x = self.position.x.round() as i32;
        let y = self.position.y.round() as i32;
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
    let mut canvas = window.into_canvas().build()?;

    let room = Room::new(20, 10);
    let mut player = Player::new();

    // Use a simple fixed 60 FPS timestep for updating the game state for now.
    // See https://gafferongames.com/post/fix_your_timestep/ for more timestep algorithms.
    let frame_duration = Duration::from_secs(1) / 60;
    let time_delta = frame_duration.as_fractional_secs() as f32;
    let mut frame_start_time = Instant::now();
    let mut frame_deadline = frame_start_time + frame_duration;

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

        player.update(time_delta);
        room.render(&mut canvas)?;
        player.render(&mut canvas)?;
        canvas.present();

        let now = Instant::now();
        let process_duration = now - frame_start_time;
        if now < frame_deadline {
            let sleep_duration = frame_deadline - now;
            eprintln!(
                "Processing frame took {}, {} ahead of deadline",
                TimeFormat(process_duration),
                TimeFormat(sleep_duration)
            );
            thread::sleep(sleep_duration);
        } else {
            eprintln!(
                "Processing frame took {}, {} behind deadline",
                TimeFormat(process_duration),
                TimeFormat(now - frame_deadline)
            );
        }
        frame_start_time = Instant::now();
        frame_deadline += frame_duration;
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
