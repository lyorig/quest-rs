use halcyon::{
    Result,
    color::Rgba,
    rect::{Point, RectF32},
    renderer::Renderer,
    resource::Resource,
    surface::Surface,
    texture::Texture,
    window::{Window, WindowBuilder},
};

use crate::chk;

pub struct Viewer {
    pub window: Window,
    renderer: Renderer,
}

impl Viewer {
    pub fn new() -> Result<Self> {
        let window = WindowBuilder::new()
            .title(c"Atlas Viewer")
            .position(Point::new(0, 0))
            .focusable(false)
            .build()?;

        let renderer = Renderer::new(window.as_ref(), None)?;

        Ok(Self { window, renderer })
    }

    pub fn update<T: Iterator<Item = RectF32>>(&self, s: Surface, rects: T) {
        let size = s.size();
        chk!(self.window.set_size(size));
        chk!(self.window.show());
        chk!(self.window.sync());

        chk!(self.renderer.clear());

        let tex = Texture::from_surface(self.renderer.as_ref(), s.as_ref()).unwrap();
        chk!(self.renderer.draw(tex.as_ref(), None, None));

        let old_col = self.renderer.xchg_draw_color_f32(Rgba::RED);
        for rect in rects {
            chk!(self.renderer.draw_rect(rect));
        }

        self.renderer.set_draw_color_f32(old_col);

        chk!(self.renderer.present());
    }
}
