use halcyon::{Result, renderer::Renderer, resource::Ref, surface::Surface, texture::Texture};

pub mod resource_loader;
pub mod scheduler;

/// In debug builds, calls [`Result::expect`] on the passed value.
/// Otherwise, the value is ignored.
///
/// This can be used as a debug-only [`Result::expect`].
///
/// # Example
/// ```rust
/// use halcyon::chk;
/// use halcyon::traits::Ref;
/// use halcyon::renderer::Renderer;
///
/// fn render(rnd: Ref<Renderer>) {
///     // this should never really fail, but just in case
///     chk!(rnd.clear());
/// }
/// ```
#[macro_export]
macro_rules! chk {
    ($res:expr) => {
        cfg_select! {
            debug_assertions => {
                $res.expect("An SDL call failed unexpectedly")
            },
            _ => {
                _ = $res
            }
        }
    };
}

/// Reads a [`Texture`]'s pixels using a [`Renderer`] into a [`Surface`].
pub fn read_pixels(rnd: Ref<Renderer>, tex: Ref<Texture>) -> Result<Surface> {
    let old_tgt = rnd.xchg_target(tex)?;
    let surface = rnd.read_target()?;

    rnd.set_target_opt(old_tgt)?;

    Ok(surface)
}
