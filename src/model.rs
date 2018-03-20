use std::time::Duration;

use failure::{err_msg, Error};
use floating_duration::TimeAsFloat;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

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
    room: Room,
}

impl Model {
    pub fn new(fps: u32) -> Model {
        let player = Player::new();
        Model {
            frame_duration: Duration::from_secs(1) / fps,
            time_since_last_tick: Duration::new(0, 0),
            player,
            old_player: player,
            room: Room::new(20, 10),
        }
    }

    pub fn left_pressed(&mut self) {
        self.player.set_horiz_state(PlayerHorizState::MovingLeft);
    }

    pub fn left_released(&mut self) {
        if self.player.horiz_state() == PlayerHorizState::MovingLeft {
            self.player
                .set_horiz_state(PlayerHorizState::StopMovingLeft);
        }
    }

    pub fn right_pressed(&mut self) {
        self.player.set_horiz_state(PlayerHorizState::MovingRight);
    }

    pub fn right_released(&mut self) {
        if self.player.horiz_state() == PlayerHorizState::MovingRight {
            self.player
                .set_horiz_state(PlayerHorizState::StopMovingRight);
        }
    }

    pub fn up_pressed(&mut self) {
        self.player.set_vert_state(PlayerVertState::Jumping);
    }

    pub fn up_released(&mut self) {
        if self.player.vert_state() == PlayerVertState::Jumping {
            self.player.set_vert_state(PlayerVertState::Falling);
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
        self.room.render(canvas)?;
        render_player.xpos += self.player.xspeed * time_delta;
        render_player.ypos += self.player.yspeed * time_delta;
        render_player.render(canvas)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Player {
    horiz_state: PlayerHorizState,
    vert_state: PlayerVertState,
    xpos: f32,
    ypos: f32,
    xspeed: f32,
    yspeed: f32,
    width: u32,
    height: u32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            horiz_state: PlayerHorizState::Idle,
            vert_state: PlayerVertState::Falling,
            xpos: 10.0,
            ypos: 10.0,
            xspeed: 0.0,
            yspeed: 0.0,
            width: 8,
            height: 20,
        }
    }

    pub fn horiz_state(&self) -> PlayerHorizState {
        self.horiz_state
    }

    pub fn set_horiz_state(&mut self, state: PlayerHorizState) {
        if self.horiz_state == state {
            return;
        }
        self.horiz_state = state;
        debug!("Player horiz state is now {:?}", self.horiz_state);
    }

    pub fn vert_state(&self) -> PlayerVertState {
        self.vert_state
    }

    pub fn set_vert_state(&mut self, state: PlayerVertState) {
        if self.vert_state == state {
            return;
        }
        self.vert_state = state;
        debug!("Player vert state is now {:?}", self.vert_state);
    }

    pub fn update(&mut self, dt: f32) {
        const WALK_SPEED: f32 = 120.0; // Maximum walk speed, in pixels per second
        const WALK_TIME: f32 = 0.2; // Time to go from 0 to `WALK_SPEED`, in seconds
        const WALK_ACCEL: f32 = WALK_SPEED / WALK_TIME;
        const STOP_TIME: f32 = 0.3; // Time to go from `WALK_SPEED` back to 0
        const STOP_ACCEL: f32 = WALK_SPEED / STOP_TIME;
        const FALL_SPEED: f32 = 300.0;
        const FALL_TIME: f32 = 1.0;
        const FALL_ACCEL: f32 = FALL_SPEED / FALL_TIME;
        const JUMP_SPEED: f32 = -120.0;
        const JUMP_TIME: f32 = 0.1;
        const JUMP_ACCEL: f32 = JUMP_SPEED / JUMP_TIME;

        let (xaccel, xminspeed, xmaxspeed) = match self.horiz_state {
            PlayerHorizState::Idle => (0.0, 0.0, 0.0),
            PlayerHorizState::MovingLeft => (-WALK_ACCEL, -WALK_SPEED, WALK_SPEED),
            PlayerHorizState::MovingRight => (WALK_ACCEL, -WALK_SPEED, WALK_SPEED),
            PlayerHorizState::StopMovingLeft => (STOP_ACCEL, -WALK_SPEED, 0.0),
            PlayerHorizState::StopMovingRight => (-STOP_ACCEL, 0.0, WALK_SPEED),
        };
        let yaccel = match self.vert_state {
            PlayerVertState::Standing => 0.0,
            PlayerVertState::Falling => FALL_ACCEL,
            PlayerVertState::Jumping => JUMP_ACCEL,
        };

        // Calculate new speed based on acceleration
        self.xspeed = (self.xspeed + xaccel * dt).min(xmaxspeed).max(xminspeed);
        self.yspeed = (self.yspeed + yaccel * dt).min(FALL_SPEED).max(JUMP_SPEED);

        // Calculate new position based on speed
        self.xpos += self.xspeed * dt;
        self.ypos += self.yspeed * dt;

        // Change horizontal state to idle when player has stopped moving
        if self.xspeed == 0.0 {
            self.set_horiz_state(PlayerHorizState::Idle);
        }

        // Change vertical state to falling when player has reached maximum jump speed
        if self.yspeed == JUMP_SPEED {
            self.set_vert_state(PlayerVertState::Falling);
        }

        // "Collision detection"
        let floor_height = 160.0 - 16.0;
        if self.ypos + self.height as f32 > floor_height {
            self.set_vert_state(PlayerVertState::Standing);
            self.yspeed = 0.0;
            self.ypos = floor_height - self.height as f32;
        }

        trace!(
            "Player accel: ({}, {}), speed: ({}, {}), pos: ({}, {})",
            xaccel,
            yaccel,
            self.xspeed,
            self.yspeed,
            self.xpos,
            self.ypos
        );
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        let x = self.xpos.round() as i32;
        let y = self.ypos.round() as i32;
        canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        canvas
            .fill_rect(Rect::new(x, y, self.width, self.height))
            .map_err(err_msg)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerHorizState {
    Idle,
    MovingLeft,
    MovingRight,
    StopMovingLeft,
    StopMovingRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerVertState {
    Standing,
    Falling,
    Jumping,
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
pub enum Tile {
    Empty,
    Filled,
}
