use std::time::{Duration, Instant};

use failure::{err_msg, Error};
use floating_duration::{TimeAsFloat, TimeFormat};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

use math::Vec2;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerState {
    Idle,
    MovingLeft,
    MovingRight,
}
