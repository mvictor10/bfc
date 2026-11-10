//! Tipos geométricos pequenos e sem dependências.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(self) -> f32 { self.x + self.w }
    pub fn bottom(self) -> f32 { self.y + self.h }

    /// Interseção com bordas exclusivas: encostar sem sobrepor não é colisão.
    pub fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }
}

#[cfg(test)]
mod tests {
    use super::Rect;

    #[test]
    fn rectangles_intersect_only_when_overlapping() {
        assert!(Rect::new(0.0, 0.0, 10.0, 10.0).intersects(Rect::new(9.0, 9.0, 10.0, 10.0)));
        assert!(!Rect::new(0.0, 0.0, 10.0, 10.0).intersects(Rect::new(10.0, 0.0, 5.0, 5.0)));
    }
}
