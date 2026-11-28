use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;

use super::super::{deferred, invalid};
use crate::{SliceError, ThumbnailData, gcode_thumbnails::encode_png};

const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const SMALL_EDGE: u32 = 128;
const MAX_EDGE: u32 = 4096;

pub(super) fn regenerate(input: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SliceError> {
    let image = decode(input)?;
    let small = downsample(&image)?;
    Ok((encode_png(&image), encode_png(&small)))
}

fn decode(input: &[u8]) -> Result<ThumbnailData, SliceError> {
    let mut chunks = input.strip_prefix(SIGNATURE).ok_or_else(invalid_png)?;
    let (b"IHDR", header) = chunk(&mut chunks)? else {
        return Err(invalid_png());
    };
    let [w0, w1, w2, w3, h0, h1, h2, h3, 8, 6, 0, 0, 0] = *header else {
        return Err(deferred(
            "expected a non-interlaced 8-bit RGBA saved plate PNG",
        ));
    };
    let width = u32::from_be_bytes([w0, w1, w2, w3]);
    let height = u32::from_be_bytes([h0, h1, h2, h3]);
    if !(1..=MAX_EDGE).contains(&width) || !(1..=MAX_EDGE).contains(&height) {
        return Err(deferred("saved plate PNG dimensions out of range"));
    }
    let mut compressed = Vec::new();
    let mut ended_data = false;
    let mut seen_palette = false;
    loop {
        match chunk(&mut chunks)? {
            (b"IDAT", bytes) if !ended_data => compressed.extend_from_slice(bytes),
            (b"IEND", []) if chunks.is_empty() && !compressed.is_empty() => break,
            (b"PLTE", bytes)
                if !seen_palette
                    && compressed.is_empty()
                    && (3..=768).contains(&bytes.len())
                    && bytes.len().is_multiple_of(3) =>
            {
                seen_palette = true;
            }
            (tag, _) if tag[0].is_ascii_lowercase() => ended_data |= !compressed.is_empty(),
            _ => return Err(invalid_png()),
        }
    }

    let stride = width as usize * 4;
    let scanline = stride + 1;
    let size = scanline * height as usize;
    let filtered =
        decompress_to_vec_zlib_with_limit(&compressed, size).map_err(|_| invalid_png())?;
    if filtered.len() != size {
        return Err(invalid_png());
    }

    // PNGReadWrite.cpp:193–195 feeds the thumbnail encoder bottom-up RGBA.
    let mut pixels = vec![0_u8; stride * height as usize];
    let empty_row = vec![0_u8; stride];
    for (y, row) in filtered.chunks_exact(scanline).enumerate() {
        let start = (height as usize - 1 - y) * stride;
        let (lower, upper) = pixels.split_at_mut(start + stride);
        let dest = &mut lower[start..];
        let above = upper.get(..stride).unwrap_or(&empty_row);
        for x in 0..stride {
            let left = x.checked_sub(4).map_or(0, |index| dest[index]);
            let upper_left = x.checked_sub(4).map_or(0, |index| above[index]);
            let predictor = match row[0] {
                0 => 0,
                1 => left,
                2 => above[x],
                3 => ((u16::from(left) + u16::from(above[x])) / 2) as u8,
                4 => paeth(left, above[x], upper_left),
                _ => return Err(invalid_png()),
            };
            dest[x] = row[x + 1].wrapping_add(predictor);
        }
    }
    ThumbnailData::new(width, height, pixels)
}

fn chunk<'a>(rest: &mut &'a [u8]) -> Result<(&'a [u8; 4], &'a [u8]), SliceError> {
    let (size, body) = rest.split_first_chunk::<4>().ok_or_else(invalid_png)?;
    let size = u32::from_be_bytes(*size) as usize;
    let checked = size.checked_add(4).ok_or_else(invalid_png)?;
    let (tag_and_data, tail) = body.split_at_checked(checked).ok_or_else(invalid_png)?;
    let (crc, next) = tail.split_first_chunk::<4>().ok_or_else(invalid_png)?;
    let (tag, data) = tag_and_data
        .split_first_chunk::<4>()
        .ok_or_else(invalid_png)?;
    if !tag.iter().all(u8::is_ascii_alphabetic)
        || !tag[2].is_ascii_uppercase()
        || crc32fast::hash(tag_and_data) != u32::from_be_bytes(*crc)
    {
        return Err(invalid_png());
    }
    *rest = next;
    Ok((tag, data))
}

fn paeth(left: u8, above: u8, upper_left: u8) -> u8 {
    let (a, b, c) = (i16::from(left), i16::from(above), i16::from(upper_left));
    let (da, db, dc) = ((b - c).abs(), (a - c).abs(), (a + b - 2 * c).abs());
    if da <= db && da <= dc {
        left
    } else if db <= dc {
        above
    } else {
        upper_left
    }
}

fn downsample(image: &ThumbnailData) -> Result<ThumbnailData, SliceError> {
    let step = image.width() / SMALL_EDGE;
    if step == 0 || step != image.height() / SMALL_EDGE {
        return Err(deferred(
            "saved plate PNG requires positive equal downsample steps",
        ));
    }
    let step = step as usize;
    let stride = image.width() as usize * 4;
    let edge = SMALL_EDGE as usize;
    let mut pixels = Vec::with_capacity(edge * edge * 4);
    // bbs_3mf.cpp:6976–7016 averages each channel and discards incomplete edge blocks.
    for y in 0..edge {
        for x in 0..edge {
            let mut channels = [0_u32; 4];
            let start = x * step * 4;
            let block = image.pixels()[y * step * stride..(y + 1) * step * stride]
                .chunks_exact(stride)
                .flat_map(|row| row[start..start + step * 4].as_chunks::<4>().0);
            for pixel in block {
                channels = std::array::from_fn(|i| channels[i] + u32::from(pixel[i]));
            }
            pixels.extend(channels.map(|sum| (sum / (step * step) as u32) as u8));
        }
    }
    ThumbnailData::new(SMALL_EDGE, SMALL_EDGE, pixels)
}

fn invalid_png() -> SliceError {
    invalid("invalid saved plate PNG")
}
