use std::time::Duration;

use failure::{err_msg, Error};
use floating_duration::TimeAsFloat;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, RenderTarget};

use room::{Room, TileKind};

const TICKS_PER_SECOND: u32 = 150;

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
    pub fn new(room: Room) -> Model {
        let player = Player::new();
        Model {
            frame_duration: Duration::from_secs(1) / TICKS_PER_SECOND,
            time_since_last_tick: Duration::new(0, 0),
            player,
            old_player: player,
            room,
        }
    }

    pub fn set_room(&mut self, room: Room) {
        self.room = room;
    }

    pub fn key_pressed(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Left => self.player.set_horiz_state(PlayerHorizState::MovingLeft),
            Keycode::Right => self.player.set_horiz_state(PlayerHorizState::MovingRight),
            Keycode::Up if self.player.vert_state() == PlayerVertState::Standing => {
                self.player.set_vert_state(PlayerVertState::Jumping)
            }
            _ => (),
        }
    }

    pub fn key_released(&mut self, keycode: Keycode) {
        match keycode {
            Keycode::Left if self.player.horiz_state() == PlayerHorizState::MovingLeft => {
                self.player
                    .set_horiz_state(PlayerHorizState::StopMovingLeft)
            }
            Keycode::Right if self.player.horiz_state() == PlayerHorizState::MovingRight => {
                self.player
                    .set_horiz_state(PlayerHorizState::StopMovingRight)
            }
            Keycode::Up if self.player.vert_state() == PlayerVertState::Jumping => {
                self.player.set_vert_state(PlayerVertState::Falling)
            }
            _ => (),
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
            self.player.update(time_delta, &self.room);
            let room_width = self.room.width() as f32 * self.room.tile_size() as f32;
            if self.player.xpos >= room_width {
                self.room = Room::default();
                self.player.xpos -= room_width;
            }
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
    width: f32,
    height: f32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            horiz_state: PlayerHorizState::Idle,
            vert_state: PlayerVertState::Falling,
            xpos: 20.0,
            ypos: 10.0,
            xspeed: 0.0,
            yspeed: 0.0,
            width: 8.0,
            height: 20.0,
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

    pub fn update(&mut self, dt: f32, room: &Room) {
        const WALK_SPEED: f32 = 120.0; // Maximum walk speed, in pixels per second
        const WALK_TIME: f32 = 0.2; // Time to go from 0 to `WALK_SPEED`, in seconds
        const WALK_ACCEL: f32 = WALK_SPEED / WALK_TIME;
        const STOP_TIME: f32 = 0.3; // Time to go from `WALK_SPEED` back to 0
        const STOP_ACCEL: f32 = WALK_SPEED / STOP_TIME;
        const FALL_SPEED: f32 = 300.0;
        const FALL_TIME: f32 = 1.0;
        const FALL_ACCEL: f32 = FALL_SPEED / FALL_TIME;
        const JUMP_SPEED: f32 = -130.0;
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

        // Stop horizontal movement when walking into a wall
        if self.xspeed > 0.0 {
            let tile1_right = room.tile_at_coord(self.xpos + self.width, self.ypos + 0.5);
            let tile2_right =
                room.tile_at_coord(self.xpos + self.width, self.ypos + self.height - 0.5);
            if (tile1_right.kind == TileKind::Filled
                && room.tile_at_index(tile1_right.x - 1, tile1_right.y).kind == TileKind::Empty)
                || (tile2_right.kind == TileKind::Filled
                    && room.tile_at_index(tile2_right.x - 1, tile2_right.y).kind == TileKind::Empty)
            {
                trace!("Player hit wall to the right");
                self.xspeed = 0.0;
                self.xpos = tile1_right.rect.left() - self.width;
            }
        } else if self.xspeed < 0.0 {
            let tile1_left = room.tile_at_coord(self.xpos, self.ypos + 0.5);
            let tile2_left = room.tile_at_coord(self.xpos, self.ypos + self.height - 0.5);
            if (tile1_left.kind == TileKind::Filled
                && room.tile_at_index(tile1_left.x + 1, tile1_left.y).kind == TileKind::Empty)
                || (tile2_left.kind == TileKind::Filled
                    && room.tile_at_index(tile2_left.x + 1, tile2_left.y).kind == TileKind::Empty)
            {
                trace!("Player hit wall to the left");
                self.xspeed = 0.0;
                self.xpos = tile1_left.rect.right();
            }
        }

        if self.yspeed >= 0.0 {
            // Handle presence or absence of floor below player
            let tile1_below = room.tile_at_coord(self.xpos + 0.5, self.ypos + self.height);
            let tile2_below =
                room.tile_at_coord(self.xpos + self.width - 0.5, self.ypos + self.height);
            match (tile1_below.kind, tile2_below.kind) {
                // Stand if either tile is filled
                (TileKind::Filled, _) | (_, TileKind::Filled) => {
                    if self.yspeed > 0.0 {
                        trace!("Player hit floor");
                        self.set_vert_state(PlayerVertState::Standing);
                        self.yspeed = 0.0;
                        self.ypos = tile1_below.rect.top() - self.height;
                    }
                }
                // Fall if standing and both tiles are empty
                (TileKind::Empty, TileKind::Empty) => {
                    if self.yspeed == 0.0 {
                        trace!("Player fall from ledge");
                        if self.vert_state == PlayerVertState::Standing {
                            self.set_vert_state(PlayerVertState::Falling);
                        }
                    }
                }
            }
        } else {
            // Stop vertical movement when hitting a ceiling
            let tile1_above = room.tile_at_coord(self.xpos + 0.5, self.ypos);
            let tile2_above = room.tile_at_coord(self.xpos + self.width - 0.5, self.ypos);
            match (tile1_above.kind, tile2_above.kind) {
                (TileKind::Filled, _) | (_, TileKind::Filled) => {
                    trace!("Player hit ceiling");
                    self.set_vert_state(PlayerVertState::Falling);
                    self.yspeed = 0.0;
                    self.ypos = tile1_above.rect.bottom();
                }
                _ => (),
            }
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
        let w = self.width.round() as u32;
        let h = self.height.round() as u32;
        canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        canvas.fill_rect(SdlRect::new(x, y, w, h)).map_err(err_msg)?;
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
