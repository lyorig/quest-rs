use halcyon::rect::PointI32;
use sdl3_sys::events::SDL_WindowEvent;

use crate::game::resources::Resources;

pub struct ResizeInfo {
    pub new_size: PointI32,
}

impl ResizeInfo {
    pub fn new(we: SDL_WindowEvent) -> Self {
        let new_size = PointI32::new(we.data1, we.data2);
        Self { new_size }
    }
}

/// A UI layer which wants to respond to canvas resize events.
pub trait Layer {
    fn resize(&mut self, info: &ResizeInfo, res: &mut Resources);
}
