use halcyon::defs::SdlResult;

pub mod lazy_static;

#[cfg(debug_assertions)]
pub fn chk(result: SdlResult) {
    result.expect("An SDL call failed unexpectedly");
}

#[cfg(not(debug_assertions))]
pub fn chk(_: SdlResult) {}
