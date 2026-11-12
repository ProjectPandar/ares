//! Orca v2.5.0 `Thumbnails.cpp::compress_thumbnail_jpg`, libjpeg-turbo3.2.0.
//! Fixed quality85 sequential YCbCr420 path. See jpg/README.ijg.
mod entropy;
mod fdct;
mod markers;
mod pixels;
mod tables;

use super::ThumbnailData;

pub(super) fn encode(image: &ThumbnailData) -> Vec<u8> {
    let planes = pixels::planes(image);
    let tables = tables::quantization();
    let quantizers = tables.map(|table| fdct::Quantizer::new(&table));
    let mut output = Vec::with_capacity(image.pixels().len() / 4);
    markers::header(&mut output, image.width(), image.height(), &tables);
    let mut entropy = entropy::Encoder::new(&mut output);
    for row in 0..(image.height() as usize).div_ceil(16) {
        for column in 0..(image.width() as usize).div_ceil(16) {
            for (component, plane) in planes.iter().enumerate() {
                component_blocks(&mut entropy, &quantizers, plane, component, (column, row));
            }
        }
    }
    entropy.finish();
    output.extend_from_slice(&[255, 217]);
    output
}

// jccoefct.c::compress_data: dummy blocks copy only the preceding DC value.
fn component_blocks(
    entropy: &mut entropy::Encoder<'_>,
    quantizers: &[fdct::Quantizer; 2],
    plane: &pixels::Plane,
    component: usize,
    (column, row): (usize, usize),
) {
    let sampling = if component == 0 { 2 } else { 1 };
    let quantizer = &quantizers[usize::from(component != 0)];
    let mut previous_dc = 0;
    for y in 0..sampling {
        for x in 0..sampling {
            let x = column * sampling + x;
            let y = row * sampling + y;
            let block = if x * 8 < plane.width && y * 8 < plane.height {
                quantizer.block(plane.block(x, y))
            } else {
                let mut dummy = [0; 64];
                dummy[0] = previous_dc;
                dummy
            };
            previous_dc = block[0];
            entropy.block(&block, component);
        }
    }
}
