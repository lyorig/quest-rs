use halcyon::rect::{PointF32, PointI32};

pub struct ResizeInfo {
    old_size: PointI32,
    new_size: PointI32,
}

impl ResizeInfo {
    pub fn ratio(&self) -> PointF32 {
        let x = self.new_size.x as f32 / self.old_size.x as f32;
        let y = self.new_size.y as f32 / self.old_size.y as f32;
        PointF32::new(x, y)
    }
}

pub trait Layer {
    fn resize(&mut self, layout: &ResizeInfo);
}
