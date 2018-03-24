use failure::{err_msg, Error};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};

use room::Room;

pub struct Editor {
    room: Room,
    cursor_x: u32,
    cursor_y: u32,
}

impl Editor {
    pub fn new(room: Room) -> Editor {
        Editor {
            room,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn room(&self) -> &Room {
        &self.room
    }

    pub fn key_pressed(&mut self, key: Keycode) {
        match key {
            Keycode::Left => self.cursor_x = self.cursor_x.saturating_sub(1),
            Keycode::Right => self.cursor_x = (self.cursor_x + 1).min(self.room.width() - 1),
            Keycode::Up => self.cursor_y = self.cursor_y.saturating_sub(1),
            Keycode::Down => self.cursor_y = (self.cursor_y + 1).min(self.room.height() - 1),
            Keycode::Space => self.room
                .toggle_tile_at_index(self.cursor_x, self.cursor_y)
                .unwrap_or_else(|error| {
                    // Cursor got out of bounds somehow, so reset it
                    error!("{}; resetting cursor", error);
                    self.cursor_x = 0;
                    self.cursor_y = 0;
                }),
            _ => (),
        }
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        self.room.render(canvas)?;
        canvas.set_draw_color(Color::RGB(0xFF, 0x00, 0x00));
        let cursor_rect = self.room
            .tile_at_index(self.cursor_x, self.cursor_y)
            .rect
            .sdl_rect();
        canvas.draw_rect(cursor_rect).map_err(err_msg)?;
        Ok(())
    }
}
