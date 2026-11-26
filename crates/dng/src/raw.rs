//! Bounded DNG raw layout and strip/tile decoding from the shared RAW library.

use crate::tiff::{Ifd, Tiff};
use crate::{Error, Result};
pub use nodalia_raw::dng_lines::{MAX_SAMPLES, PHOTOMETRIC_CFA, PHOTOMETRIC_LINEAR_RAW, RawImage, RawLayout};

/// Unpack `w` × `h` × `spp` uncompressed samples of `bits` bits, as the shared decoder does.
pub fn unpack(bytes: &[u8], w: usize, h: usize, spp: usize, bits: u32, big_endian: bool) -> Result<Vec<u16>> {
    nodalia_raw::dng_lines::unpack(bytes, w, h, spp, bits, big_endian).map_err(error)
}

pub(crate) fn error(e: nodalia_raw::dng_lines::Error) -> Error {
    match e {
        nodalia_raw::dng_lines::Error::NotTiff => Error::NotDng,
        nodalia_raw::dng_lines::Error::Truncated => Error::Truncated,
        nodalia_raw::dng_lines::Error::TooLarge => Error::TooLarge,
        nodalia_raw::dng_lines::Error::Malformed(s) => Error::Malformed(s),
        nodalia_raw::dng_lines::Error::Unsupported(s) => Error::Unsupported(s),
    }
}

/// Find the largest full-resolution raw IFD.
pub fn find_raw_ifd<'i>(t: &Tiff, ifds: &'i [Ifd]) -> Result<&'i Ifd> {
    nodalia_raw::dng_lines::find_raw_ifd(&t.inner(), ifds).map_err(error)
}
/// Read and validate a raw IFD's layout.
pub fn layout(t: &Tiff, ifd: &Ifd) -> Result<RawLayout> {
    nodalia_raw::dng_lines::layout(&t.inner(), ifd).map_err(error)
}
/// Decode the raw samples of an IFD.
pub fn read(t: &Tiff, ifd: &Ifd) -> Result<RawImage> {
    nodalia_raw::dng_lines::read(&t.inner(), ifd).map_err(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_keeps_its_public_signature_and_errors() {
        assert_eq!(unpack(&[0x12, 0x34], 2, 1, 1, 8, true).unwrap(), [0x12, 0x34]);
        assert!(matches!(unpack(&[0x12], 1, 1, 1, 0, false), Err(Error::Malformed(_))));
    }
}
