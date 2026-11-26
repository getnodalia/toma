//! Compatibility facade for the shared `nodalia-cfb` reader and writer.
//! AAF application mapping stays in `toma-interchange`; all CFB format logic lives upstream.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable))]

pub use nodalia_cfb::*;

#[cfg(test)]
mod tests;
