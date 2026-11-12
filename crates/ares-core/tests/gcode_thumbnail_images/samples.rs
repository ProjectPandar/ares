pub struct Frame {
    pub size: [f64; 2],
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn asymmetric() -> Frame {
    let (width, height) = (32, 24);
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&[
                ((x * 7 + y * 3) % 256) as u8,
                ((x * 3 + y * 11) % 256) as u8,
                ((x * y + 19) % 256) as u8,
                [0, 1, 127, 255][((x + y) % 4) as usize],
            ]);
        }
    }
    Frame {
        size: [32.0, 24.0],
        width,
        height,
        pixels,
    }
}

pub fn families() -> Vec<(&'static str, Frame)> {
    let solid = Frame {
        size: [32.0, 24.0],
        width: 32,
        height: 24,
        pixels: [83, 139, 197, 255].repeat(32 * 24),
    };
    let mut colors = Vec::with_capacity(64 * 48 * 4);
    for color in 0_u32..64 * 48 {
        colors.extend_from_slice(&[
            (((color >> 11) & 31) << 3) as u8,
            (((color >> 5) & 63) << 2) as u8,
            ((color & 31) << 3) as u8,
            255,
        ]);
    }
    let high_color = Frame {
        size: [64.0, 48.0],
        width: 64,
        height: 48,
        pixels: colors,
    };
    // Thumbnails.cpp:234: these RGB565 values become zero in BTT output.
    let special_colors = [
        [0, 4, 0, 255],
        [8, 8, 8, 255],
        [8, 12, 8, 255],
        [0, 0, 0, 255],
        [255, 255, 255, 0],
        [255, 255, 255, 255],
    ];
    let pixels = (0..32 * 24)
        .flat_map(|index| special_colors[index % special_colors.len()])
        .collect();
    let special = Frame {
        size: [32.0, 24.0],
        width: 32,
        height: 24,
        pixels,
    };
    vec![
        ("asymmetric-alpha", asymmetric()),
        ("solid-runs", solid),
        ("high-color-ties", high_color),
        ("btt-special-colors", special),
    ]
}
