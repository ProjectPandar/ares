use super::ThumbnailData;

/// `compress_thumbnail_btt_tft` and its `export_thumbnails_to_file` layout.
pub(super) fn append(image: &ThumbnailData, output: &mut Vec<u8>, last_definition: bool) {
    output
        .extend_from_slice(format!(";{:04x}{:04x}\r\n", image.width(), image.height()).as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let row_bytes = image.width() as usize * 4;
    for row in image.pixels().chunks_exact(row_bytes).rev() {
        output.push(b';');
        for pixel in row.as_chunks::<4>().0 {
            let alpha = u16::from(pixel[3]);
            let red = u16::from(pixel[0]) * alpha / 255;
            let green = u16::from(pixel[1]) * alpha / 255;
            let blue = u16::from(pixel[2]) * alpha / 255;
            let color = (red >> 3) << 11 | (green >> 2) << 5 | (blue >> 3);
            let color = match color {
                0x0020 | 0x0841 | 0x0861 => 0,
                color => color,
            };
            output.extend_from_slice(&[
                HEX[usize::from(color >> 12)],
                HEX[usize::from((color >> 8) & 15)],
                HEX[usize::from((color >> 4) & 15)],
                HEX[usize::from(color & 15)],
            ]);
        }
        output.extend_from_slice(b"\r\n");
    }
    if last_definition {
        output.extend_from_slice(b"; bigtree thumbnail end\r\n\r\n");
    }
}
