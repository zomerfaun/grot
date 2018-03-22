//! Geometry stuff

use sdl2::rect::Rect as SdlRect;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        debug_assert!(w > 0.0);
        debug_assert!(h > 0.0);
        Rect { x, y, w, h }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn sdl_rect(&self) -> SdlRect {
        SdlRect::new(
            self.x.round() as i32,
            self.y.round() as i32,
            self.w.round() as u32,
            self.h.round() as u32,
        )
    }
}
