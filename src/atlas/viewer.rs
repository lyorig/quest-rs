use halcyon::{
    Result,
    color::Rgba,
    gpu::Device,
    properties::Properties,
    rect::{Point, RectF32},
    renderer::Renderer,
    resource::Resource,
    surface::Surface,
    texture::Texture,
    window::Window,
};

use crate::chk;

pub struct Viewer {
    /// These fields must drop in this order: the renderer releases the
    /// window from the device, then the window drops, then the device drops.
    renderer: Renderer,
    pub window: Window,
    device: Device,
}

impl Viewer {
    pub fn new() -> Result<Self> {
        let props = Properties::global()?;
        let wnd = Window::builder(props)
            .title(c"Atlas Viewer")
            .position(Point::new(0, 0))
            .focusable(false)
            .build()?;

        let device = Device::builder(props)
            .debug_mode(false)
            .prefer_low_power(true)
            .shaders_metallib(true)
            .shaders_dxil(true)
            .build_cleanup()?;

        let renderer = Renderer::new_gpu(device.as_ref(), wnd.as_ref())?;

        Ok(Self {
            renderer,
            window: wnd,
            device,
        })
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
