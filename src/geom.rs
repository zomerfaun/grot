//! Geometry stuff

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

    pub fn top(&self) -> f32 {
        self.y
    }
}
