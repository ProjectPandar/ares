use super::ThumbnailData;
use miniz_oxide::deflate::core::{
    CompressorOxide, TDEFLFlush, compress_to_output, deflate_flags::TDEFL_WRITE_ZLIB_HEADER,
};

/// `miniz.c::tdefl_write_image_to_png_file_in_memory_ex`: level 6, RGBA, flip.
pub(super) fn encode(image: &ThumbnailData) -> Vec<u8> {
    let mut output = Vec::with_capacity(57 + image.pixels().len() + image.height() as usize);
    output.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    output.extend_from_slice(&13_u32.to_be_bytes());
    output.extend_from_slice(b"IHDR");
    output.extend_from_slice(&image.width().to_be_bytes());
    output.extend_from_slice(&image.height().to_be_bytes());
    output.extend_from_slice(&[8, 6, 0, 0, 0]);
    output.extend_from_slice(&crc32fast::hash(&output[12..29]).to_be_bytes());
    let length_offset = output.len();
    output.extend_from_slice(&0_u32.to_be_bytes());
    output.extend_from_slice(b"IDAT");
    let data_offset = output.len();
    let mut compressor = CompressorOxide::new(128 | TDEFL_WRITE_ZLIB_HEADER);
    let row_bytes = image.width() as usize * 4;
    for row in image.pixels().chunks_exact(row_bytes).rev() {
        for input in [&[0_u8][..], row] {
            compress_to_output(&mut compressor, input, TDEFLFlush::None, |bytes| {
                output.extend_from_slice(bytes);
                true
            });
        }
    }
    compress_to_output(&mut compressor, &[], TDEFLFlush::Finish, |bytes| {
        output.extend_from_slice(bytes);
        true
    });
    // Orca's bundled miniz.c:1252-1253 emits a fixed zlib header at every level.
    output[data_offset..data_offset + 2].copy_from_slice(&[0x78, 0x01]);
    let data_length = (output.len() - data_offset) as u32;
    output[length_offset..length_offset + 4].copy_from_slice(&data_length.to_be_bytes());
    output.extend_from_slice(&crc32fast::hash(&output[length_offset + 4..]).to_be_bytes());
    output.extend_from_slice(b"\0\0\0\0IEND\xae\x42\x60\x82");
    output
}
