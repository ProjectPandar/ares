use super::ThumbnailData;

/// `Thumbnails.cpp::compress_thumbnail_qoi` and `deps_src/qoi/qoi.h::qoi_encode`.
pub(super) fn encode(image: &ThumbnailData) -> Vec<u8> {
    let mut output = Vec::with_capacity(image.pixels().len() / 4 * 5 + 22);
    output.extend_from_slice(b"qoif");
    output.extend_from_slice(&image.width().to_be_bytes());
    output.extend_from_slice(&image.height().to_be_bytes());
    output.extend_from_slice(&[4, 0]);
    let mut index = [[0_u8; 4]; 64];
    let mut previous = [0, 0, 0, 255];
    let mut run = 0_u8;
    let row_bytes = image.width() as usize * 4;
    let pixels = image
        .pixels()
        .chunks_exact(row_bytes)
        .rev()
        .flat_map(|row| row.as_chunks::<4>().0.iter());
    let last = image.pixels().len() / 4 - 1;
    for (position, &pixel) in pixels.enumerate() {
        let previous = std::mem::replace(&mut previous, pixel);
        if pixel == previous {
            run += 1;
            if run == 62 || position == last {
                output.push(0xc0 | (run - 1));
                run = 0;
            }
            continue;
        }
        if run > 0 {
            output.push(0xc0 | (run - 1));
            run = 0;
        }
        let hash = (usize::from(pixel[0]) * 3
            + usize::from(pixel[1]) * 5
            + usize::from(pixel[2]) * 7
            + usize::from(pixel[3]) * 11)
            % 64;
        if index[hash] == pixel {
            output.push(hash as u8);
            continue;
        }
        index[hash] = pixel;
        if pixel[3] != previous[3] {
            output.push(0xff);
            output.extend_from_slice(&pixel);
            continue;
        }
        // qoi_encode narrows deltas to signed char before its opcode tests.
        let red = pixel[0].wrapping_sub(previous[0]).cast_signed();
        let green = pixel[1].wrapping_sub(previous[1]).cast_signed();
        let blue = pixel[2].wrapping_sub(previous[2]).cast_signed();
        let red_green = red.wrapping_sub(green);
        let blue_green = blue.wrapping_sub(green);
        if (-2..=1).contains(&red) && (-2..=1).contains(&green) && (-2..=1).contains(&blue) {
            output
                .push(0x40 | ((red + 2) as u8) << 4 | ((green + 2) as u8) << 2 | (blue + 2) as u8);
        } else if (-8..=7).contains(&red_green)
            && (-32..=31).contains(&green)
            && (-8..=7).contains(&blue_green)
        {
            output.push(0x80 | (green + 32) as u8);
            output.push(((red_green + 8) as u8) << 4 | (blue_green + 8) as u8);
        } else {
            output.extend_from_slice(&[0xfe, pixel[0], pixel[1], pixel[2]]);
        }
    }
    output.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    output
}
