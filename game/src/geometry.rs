use bevy::math::{Dir2, Rect};

pub trait RectExt {
    fn radius(&self, direction: Dir2) -> f32;
}

impl RectExt for Rect {
    fn radius(&self, direction: Dir2) -> f32 {
        let theta = (direction.y / direction.x).atan();
        if theta.tan().abs() <= self.height() / self.width() {
            self.half_size().x / theta.cos().abs()
        } else {
            self.half_size().y / theta.sin().abs()
        }
    }
}
