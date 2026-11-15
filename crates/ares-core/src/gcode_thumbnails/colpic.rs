use super::ThumbnailData;
use crate::SliceError;
use std::collections::BTreeMap;

pub(super) fn append(
    image: &ThumbnailData,
    output: &mut Vec<u8>,
    first: bool,
) -> Result<(), SliceError> {
    let encoded = encode(image)?;
    let label = if first { "gimage" } else { "simage" };
    output.extend_from_slice(format!("\n\n;{label}:").as_bytes());
    output.extend_from_slice(&encoded);
    output.extend_from_slice(b"\n\n");
    Ok(())
}

/// `Thumbnails.cpp:140-192::compress_thumbnail_colpic`.
fn encode(image: &ThumbnailData) -> Result<Vec<u8>, SliceError> {
    let (mut width, mut height) = (image.width(), image.height());
    if width > 512 || height > 512 {
        let aspect_ratio = f64::from(width) / f64::from(height);
        if aspect_ratio > 1.0 {
            width = 512;
            height = (512.0 / aspect_ratio) as u32;
        } else {
            height = 512;
            width = (512.0 * aspect_ratio) as u32;
        }
    }
    if width == 0 || height == 0 {
        return Err(SliceError::InvalidInput(
            "ColPic thumbnail size cap produces an empty image".into(),
        ));
    }
    // Native copies a contiguous capped prefix, not source-width-strided rows.
    let pixels = &image.pixels()[..width as usize * height as usize * 4];
    let (pixels, _) = pixels.as_chunks::<4>();
    let colors = pixels
        .chunks_exact(width as usize)
        .rev()
        .flatten()
        .map(|&[r, g, b, a]| {
            let (r, g, b) = if a == 0 { (46, 51, 72) } else { (r, g, b) };
            (u16::from(r >> 3) << 11) | (u16::from(g >> 2) << 5) | u16::from(b >> 3)
        })
        .collect::<Vec<_>>();
    let mut binary = encode_binary(&colors, width, height);
    // ColPic_EncodeStr always pads, including an extra group at a group boundary.
    binary.extend(std::iter::repeat_n(0, 3 - binary.len() % 3));
    let mut output = Vec::with_capacity(binary.len() / 3 * 4);
    for &[a, b, c] in binary.as_chunks::<3>().0 {
        for digit in [
            a >> 2,
            (a & 3) << 4 | b >> 4,
            (b & 15) << 2 | c >> 6,
            c & 63,
        ] {
            let ascii = digit + 48;
            output.push(if ascii == b'\\' { b'~' } else { ascii });
        }
    }
    Ok(output)
}

/// `ADList0`, `ColPicEncode` and `Byte8bitEncode` (`Thumbnails.cpp:330-489`).
fn encode_binary(colors: &[u16], width: u32, height: u32) -> Vec<u8> {
    let mut palette = Vec::<(u16, u32)>::new();
    let mut slots = BTreeMap::<u16, usize>::new();
    for &color in colors {
        // ADList0 checks capacity before lookup: all counts freeze at capacity.
        if palette.len() == 1024 {
            break;
        }
        if let Some(&index) = slots.get(&color) {
            palette[index].1 += 1;
        } else {
            slots.insert(color, palette.len());
            palette.push((color, 1));
        }
    }
    // Native insertion uses >=: the later first-seen color wins a frequency tie.
    palette.reverse();
    palette.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    slots.clear();
    for (index, &(color, _)) in palette.iter().enumerate() {
        slots.insert(color, index);
    }
    let mut output = vec![0; 32];
    output[0] = 3;
    output[4..8].copy_from_slice(&width.to_le_bytes());
    output[8..12].copy_from_slice(&height.to_le_bytes());
    output[12..16].copy_from_slice(&0x05dd_c33c_u32.to_le_bytes());
    output[16..20].copy_from_slice(&(palette.len() as u32 * 2).to_le_bytes());
    for &(color, _) in &palette {
        output.extend_from_slice(&color.to_le_bytes());
    }
    let run_start = output.len();
    let mut position = 0;
    let mut last_page = 0;
    while position < colors.len() {
        let color = colors[position];
        let count = colors[position..]
            .iter()
            .take(255)
            .take_while(|&&next| next == color)
            .count();
        // Unlisted colors use palette index zero, as Byte8bitEncode's temp = 0.
        let index = slots.get(&color).copied().unwrap_or(0);
        let page = (index / 32) as u8;
        let slot = (index % 32) as u8;
        if last_page != page {
            output.push(0xe0 | page);
            last_page = page;
        }
        if count <= 6 {
            output.push((count as u8) << 5 | slot);
        } else {
            output.extend_from_slice(&[slot, count as u8]);
        }
        position += count;
    }
    let run_size = (output.len() - run_start) as u32;
    output[20..24].copy_from_slice(&run_size.to_le_bytes());
    output
}
