//! Trait extensions for making working with Vizia even nicer

use glam::Vec2;
use vizia::*;

pub trait BoundingBoxExt {
    fn center(&self) -> Vec2;
    fn center_x(&self) -> f32 {
        self.center().x
    }
    fn center_y(&self) -> f32 {
        self.center().y
    }
    fn left(&self) -> f32;
    fn right(&self) -> f32;
    fn bottom_left(&self) -> Vec2 {
        Vec2::new(self.left(), self.bottom())
    }
    fn bottom_right(&self) -> Vec2 {
        Vec2::new(self.right(), self.bottom())
    }
    fn top_left(&self) -> Vec2 {
        Vec2::new(self.left(), self.top())
    }
    fn top_right(&self) -> Vec2 {
        Vec2::new(self.right(), self.top())
    }
    fn top(&self) -> f32;
    fn bottom(&self) -> f32;
    fn shrink(&self, amount: f32) -> BoundingBox;
    fn expand(&self, amount: f32) -> BoundingBox;
    fn shrink_x(&self, amount: f32) -> BoundingBox;
    fn expand_x(&self, amount: f32) -> BoundingBox;
    fn shrink_y(&self, amount: f32) -> BoundingBox;
    fn expand_y(&self, amount: f32) -> BoundingBox;
    fn map_ui_point(&self, point: Vec2) -> Vec2;
    fn map_data_point(&self, point: Vec2) -> Vec2;
}

impl BoundingBoxExt for BoundingBox {
    fn center(&self) -> Vec2 {
        Vec2::new(self.x + (self.w / 2f32), self.y + (self.h / 2f32))
    }
    fn left(&self) -> f32 {
        self.x
    }
    fn right(&self) -> f32 {
        self.x + self.w
    }
    fn top(&self) -> f32 {
        self.y
    }
    fn bottom(&self) -> f32 {
        self.y + self.h
    }
    fn shrink(&self, amount: f32) -> Self {
        BoundingBox {
            x: self.x + amount,
            y: self.y + amount,
            w: self.w - (amount * 2f32),
            h: self.h - (amount * 2f32),
        }
    }
    fn expand(&self, amount: f32) -> Self {
        BoundingBox {
            x: self.x - amount,
            y: self.y - amount,
            w: self.w + (amount * 2f32),
            h: self.h + (amount * 2f32),
        }
    }

    fn shrink_x(&self, amount: f32) -> BoundingBox {
        let new = self.shrink(amount);
        BoundingBox {
            x: new.x,
            w: new.w,
            ..*self
        }
    }

    fn expand_x(&self, amount: f32) -> BoundingBox {
        let new = self.expand(amount);
        BoundingBox {
            x: new.x,
            w: new.w,
            ..*self
        }
    }

    fn shrink_y(&self, amount: f32) -> BoundingBox {
        let new = self.shrink(amount);
        BoundingBox {
            y: new.y,
            h: new.h,
            ..*self
        }
    }

    fn expand_y(&self, amount: f32) -> BoundingBox {
        let new = self.expand(amount);
        BoundingBox {
            y: new.y,
            h: new.h,
            ..*self
        }
    }

    /// Maps a UI point to a normalized `Vec2` from `(-1,-1)..=(1,1)`
    fn map_ui_point(&self, point: Vec2) -> Vec2 {
        // clamp point to rect bounds
        let point = point.clamp(self.top_left(), self.bottom_right());
        // local space for the point
        let point = point - self.top_left();
        Vec2::new(point.x / self.w, point.y / self.h)
    }

    /// Maps a normalized data point to absolute UI coordinates from `(-1,-1)..=(1,1)`
    fn map_data_point(&self, point: Vec2) -> Vec2 {
        let x = (point.x * self.w) + self.left();
        let y = (point.y * self.h) + self.top();
        Vec2::new(x, y)
    }
}

pub trait StyleExt {
    fn background_color(&self, cx: &Context) -> Color;
    fn border_color(&self, cx: &Context) -> Color;
}

impl StyleExt for Style {
    fn background_color(&self, cx: &Context) -> Color {
        self.background_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default()
    }
    fn border_color(&self, cx: &Context) -> Color {
        self.border_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn rect() -> BoundingBox {
        BoundingBox {
            x: 100f32,
            y: 100f32,
            w: 100f32,
            h: 100f32,
        }
    }

    #[test]
    fn get_center() {
        let rect = rect();
        assert_eq!(rect.center(), Vec2::new(150f32, 150f32));
    }

    #[test]
    fn get_left() {
        let rect = rect();
        assert_eq!(rect.left(), 100f32);
    }

    #[test]
    fn get_right() {
        let rect = rect();
        assert_eq!(rect.right(), 200f32);
    }

    #[test]
    fn get_top() {
        let rect = rect();
        assert_eq!(rect.top(), 100f32);
    }

    #[test]
    fn get_bottom() {
        let rect = rect();
        assert_eq!(rect.bottom(), 200f32);
    }

    #[test]
    fn get_shrunken() {
        let rect = rect();
        let a = rect.shrink(25f32);
        let b = BoundingBox {
            x: 125f32,
            y: 125f32,
            w: 50f32,
            h: 50f32,
        };
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.h, b.h);
        assert_eq!(a.w, b.w);
    }

    #[test]
    fn get_expanded() {
        let rect = rect();
        let a = rect.expand(25f32);
        let b = BoundingBox {
            x: 75f32,
            y: 75f32,
            w: 150f32,
            h: 150f32,
        };
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
        assert_eq!(a.h, b.h);
        assert_eq!(a.w, b.w);
    }

    #[test]
    fn get_mapped_ui_point() {
        let rect = rect();
        let cursor = Vec2::new(150f32, 150f32);
        assert_eq!(rect.map_ui_point(cursor), Vec2::splat(0.5));
    }

    #[test]
    fn get_mapped_data_point() {
        let rect = rect();
        let data = Vec2::splat(0.5);
        assert_eq!(rect.map_data_point(data), Vec2::new(150f32, 150f32));
    }
}
