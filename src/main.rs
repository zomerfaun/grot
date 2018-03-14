extern crate failure;
extern crate floating_duration;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerState {
    Idle,
    MovingLeft,
    MovingRight,
}

#[derive(Clone, Copy)]
pub struct Player {
    state: PlayerState,
    position: Vec2,
    speed: Vec2,
    acceleration: Vec2,
    start_time: Option<Instant>,
}

impl Player {
    pub fn new() -> Player {
        Player {
            state: PlayerState::Idle,
            position: Vec2::new(10.0, 10.0),
            speed: Vec2::default(),
            acceleration: Vec2::default(),
            start_time: None,
        }
    }

    pub fn state(&self) -> PlayerState {
        self.state
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
        debug!("Player state is now {:?}", self.state);
    }

    pub fn update(&mut self, dt: f32) {
        const MAX_SPEED: f32 = 120.0; // Maximum speed, in pixels per second
        const ACCELERATION_TIME: f32 = 0.2; // Time to accelerate from 0 to `MAX_SPEED`, in seconds
        const ACCELERATION_FACTOR: f32 = MAX_SPEED / ACCELERATION_TIME;

        let target_speed = match self.state {
            PlayerState::Idle => 0.0,
            PlayerState::MovingLeft => -MAX_SPEED,
            PlayerState::MovingRight => MAX_SPEED,
        };
        self.acceleration.x = if self.speed.x != target_speed {
            (target_speed - self.speed.x).signum() * ACCELERATION_FACTOR
        } else {
            0.0
        };

        // If the new speed would overshoot the target speed, recompute the
        // acceleration so that the new speed will exactly equal the target
        // speed instead.
        if (self.speed.x - target_speed).signum()
            != (self.speed.x + self.acceleration.x * dt - target_speed).signum()
        {
            debug!("Correcting acceleration because of target speed overshoot");
            self.acceleration.x = (target_speed - self.speed.x) / dt;
        }

        let old_speed = self.speed;

        // Semi-implicit Euler integration
        // See https://gafferongames.com/post/integration_basics/
        self.speed += self.acceleration * dt;
        self.position += self.speed * dt;
        trace!(
            "Player accel: {:?}, speed: {:?}, pos: {:?}",
            self.acceleration,
            self.speed,
            self.position
        );

        // I didn't believe my ACCELERATION_TIME constant above worked as intended
        // (the acceleration time seemed longer than it should be to me), so I
        // measured it. Luckily it is in fact working as intended.
        if old_speed.x == 0.0 && self.speed.x != 0.0 && self.start_time.is_none() {
            debug!("Acceleration start");
            self.start_time = Some(Instant::now());
        } else if let Some(start_time) = self.start_time {
            if self.speed.x.abs() == MAX_SPEED {
                debug!(
                    "Reached max speed in {:#}",
                    TimeFormat(start_time.elapsed())
                );
                self.start_time = None;
            }
        }
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        let x = self.position.x.round() as i32;
        let y = self.position.y.round() as i32;
        canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        canvas.fill_rect(Rect::new(x, y, 8, 20)).map_err(err_msg)?;
        Ok(())
    }
}

/// Game model.
/// 
/// The `Model` can update at a stable frame rate that is independent from
/// that of the main loop, and render at any time by interpolating object
/// positions.
/// Based on a method from <https://gafferongames.com/post/fix_your_timestep/>.
pub struct Model {
    frame_duration: Duration,
    time_since_last_tick: Duration,
    player: Player,
    old_player: Player,
}

impl Model {
    pub fn new(fps: u32) -> Model {
        let player = Player::new();
        Model {
            frame_duration: Duration::from_secs(1) / fps,
            time_since_last_tick: Duration::default(),
            player,
            old_player: player,
        }
    }

    pub fn left_pressed(&mut self) {
        self.player.set_state(PlayerState::MovingLeft);
    }

    pub fn left_released(&mut self) {
        if self.player.state() == PlayerState::MovingLeft {
            self.player.set_state(PlayerState::Idle);
        }
    }

    pub fn right_pressed(&mut self) {
        self.player.set_state(PlayerState::MovingRight);
    }

    pub fn right_released(&mut self) {
        if self.player.state() == PlayerState::MovingRight {
            self.player.set_state(PlayerState::Idle);
        }
    }

    pub fn update(&mut self, time_passed: Duration) {
        // what happens with inputs
        // when the game fps is higher than the model fps?
        // answer: all inputs SHOULD BE processed at the start of a new model tick
        self.time_since_last_tick += time_passed;
        while self.time_since_last_tick >= self.frame_duration {
            self.time_since_last_tick -= self.frame_duration;
            let time_delta = self.frame_duration.as_fractional_secs() as f32;
            self.old_player = self.player;
            self.player.update(time_delta);
        }
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        let mut render_player = self.old_player;
        let time_delta = self.time_since_last_tick.as_fractional_secs() as f32;
        render_player.position += self.player.speed * time_delta;
        render_player.render(canvas)
    }
}

fn run() -> Result<(), Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video = sdl.video().map_err(err_msg)?;
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    let window = video.window("Grot", 640, 480).build()?;
    let mut canvas = window.into_canvas().build()?;

    let room = Room::new(20, 10);
    let mut model = Model::new(60);

    let frame_duration = Duration::from_secs(1) / 60;
    let mut frame_start_time = Instant::now();
    let mut frame_deadline = frame_start_time + frame_duration;

    debug!("Running main loop");
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    debug!("Quitting");
                    return Ok(());
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    repeat: false,
                    ..
                } => {
                    // Toggle fullscreen state
                    let window = canvas.window_mut();
                    let new_fullscreen_state = match window.fullscreen_state() {
                        FullscreenType::Off => FullscreenType::Desktop,
                        _ => FullscreenType::Off,
                    };
                    debug!("New fullscreen state: {:?}", new_fullscreen_state);
                    window
                        .set_fullscreen(new_fullscreen_state)
                        .map_err(err_msg)?;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    repeat: false,
                    ..
                } => model.left_pressed(),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => model.left_released(),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    repeat: false,
                    ..
                } => model.right_pressed(),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => model.right_released(),

                _ => trace!("Unhandled event of type {:?}", event),
            }
        }

        model.update(frame_duration);
        room.render(&mut canvas)?;
        model.render(&mut canvas)?;
        canvas.present();

        let now = Instant::now();
        let process_duration = now - frame_start_time;
        if now < frame_deadline {
            let sleep_duration = frame_deadline - now;
            trace!(
                "Processing frame took {}, {} ahead of deadline",
                TimeFormat(process_duration),
                TimeFormat(sleep_duration)
            );
            thread::sleep(sleep_duration);
        } else {
            let lateness = now - frame_deadline;
            trace!(
                "Processing frame took {}, {} behind deadline",
                TimeFormat(process_duration),
                TimeFormat(lateness)
            );
            if lateness > Duration::from_secs(1) {
                warn!("Frame is {} late; resetting deadline", TimeFormat(lateness));
                frame_deadline = now;
            }
        }
        frame_start_time = Instant::now();
        frame_deadline += frame_duration;
    }
}

fn main() {
    pretty_env_logger::init();
    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        for cause in error.causes().skip(1) {
            eprintln!("Cause: {}", cause);
        }
        eprintln!("{}", error.backtrace());
        std::process::exit(1);
    }
}
