use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use failure::{err_msg, Error};
use sdl2::pixels::Color;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, RenderTarget};

use geom::Rect;

#[derive(Clone, Deserialize, Serialize)]
pub struct Room {
    width: u32,
    height: u32,
    tiles: Vec<TileKind>,
    tile_size: u32,
}

impl Room {
    pub fn new(width: u32, height: u32, tile_size: u32) -> Room {
        // Construct an empty roomsworth of tiles
        let mut tiles = vec![TileKind::Empty; (width * height) as usize];

        // Add a floor
        for x in 0..width as usize {
            tiles[(width * (height - 1)) as usize + x] = TileKind::Filled;
        }

        Room {
            width,
            height,
            tiles,
            tile_size,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tile_size(&self) -> u32 {
        self.tile_size
    }

    pub fn tile_at_index(&self, x: u32, y: u32) -> Tile {
        let kind = if x < self.width && y < self.height {
            self.tiles[self.width as usize * y as usize + x as usize]
        } else {
            TileKind::Empty
        };
        let rect = Rect::new(
            (x * self.tile_size) as f32,
            (y * self.tile_size) as f32,
            self.tile_size as f32,
            self.tile_size as f32,
        );
        Tile { x, y, kind, rect }
    }

    pub fn tile_at_coord(&self, x: f32, y: f32) -> Tile {
        self.tile_at_index(x as u32 / self.tile_size, y as u32 / self.tile_size)
    }

    pub fn toggle_tile_at_index(&mut self, x: u32, y: u32) -> Result<(), Error> {
        ensure!(
            x < self.width && y < self.height,
            "Tile index ({}, {}) out of bounds for room dimensions {}×{}",
            x,
            y,
            self.width,
            self.height
        );
        let kind = &mut self.tiles[(self.width * y) as usize + x as usize];
        *kind = match *kind {
            TileKind::Empty => TileKind::Filled,
            TileKind::Filled => TileKind::Empty,
        };
        Ok(())
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) -> Result<(), Error> {
        canvas.set_logical_size(self.width * self.tile_size, self.height * self.tile_size)?;
        canvas.set_draw_color(Color::RGB(0x20, 0x20, 0x20));
        canvas.clear();
        for (i, tile) in self.tiles.iter().enumerate() {
            let x = i as i32 % self.width as i32 * self.tile_size as i32;
            let y = i as i32 / self.width as i32 * self.tile_size as i32;
            let tile_color = match *tile {
                TileKind::Empty => Color::RGB(0x00, 0x00, 0x00),
                TileKind::Filled => Color::RGB(0x80, 0x80, 0x80),
            };
            canvas.set_draw_color(tile_color);
            canvas
                .fill_rect(SdlRect::new(x, y, self.tile_size, self.tile_size))
                .map_err(err_msg)?;
        }
        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        ::serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Room, Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let room: Room = ::serde_json::from_reader(reader)?;
        ensure!(!room.tiles.is_empty(), "Tiles data should not be empty");
        ensure!(
            room.tiles.len() == room.width as usize * room.height as usize,
            "Invalid tiles length {}; should be {} for room dimensions {}×{}",
            room.tiles.len(),
            room.width * room.height,
            room.width,
            room.height
        );
        Ok(room)
    }
}

impl Default for Room {
    fn default() -> Room {
        Room::new(20, 10, 16)
    }
}

pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub kind: TileKind,
    pub rect: Rect,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
pub enum TileKind {
    Empty,
    Filled,
}
