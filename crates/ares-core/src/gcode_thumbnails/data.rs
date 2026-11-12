use crate::SliceError;

/// `GCode/ThumbnailData.hpp::ThumbnailData`: bottom-up, unpremultiplied RGBA8.
#[derive(Debug)]
pub struct ThumbnailData {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl ThumbnailData {
    /// Takes ownership of a complete caller-rendered image without copying pixels.
    ///
    /// # Errors
    /// Returns an invalid-input error for zero dimensions, size overflow or a
    /// pixel buffer whose length differs from `width * height * 4`.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, SliceError> {
        let length = usize::try_from(width)
            .ok()
            .zip(usize::try_from(height).ok())
            .and_then(|(width, height)| width.checked_mul(height))
            .and_then(|length| length.checked_mul(4));
        if width == 0 || height == 0 || length != Some(pixels.len()) {
            return Err(SliceError::InvalidInput(
                "thumbnail requires positive dimensions and exactly width * height * 4 RGBA bytes"
                    .into(),
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
