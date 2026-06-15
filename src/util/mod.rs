use halcyon::{
    defs::SdlResult, renderer::Renderer, surface::Surface, texture::Texture, traits::Ref,
};

pub mod lazy_static;

/// Used to wrap [`SdlResult`]s on whose values you don't necessarily depend,
/// but would still like a sanity check in debug builds.
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
pub fn read_pixels(rnd: Ref<Renderer>, tex: Ref<Texture>) -> SdlResult<Surface> {
    let old_tgt = rnd.xchg_target(tex)?;
    let surface = rnd.read_target()?;

    rnd.set_target_opt(old_tgt)?;

    Ok(surface)
}
