use std::io::Cursor;

use zip::ZipArchive;

use super::{PLATE_GCODE, invalid};
use crate::SliceError;

/// The ordinary memory members use miniz's UTF-8 flag; the ASCII staged
/// G-code name does not (`miniz.c:6152,6757`, `bbs_3mf.cpp:9038`). The
/// streaming writer supplies the data descriptors and owns their offsets.
pub(super) fn write_native_fields(bytes: &mut [u8]) -> Result<(), SliceError> {
    let mut archive = ZipArchive::new(Cursor::new(&*bytes)).map_err(|e| invalid(e.to_string()))?;
    let records = (0..archive.len())
        .map(|index| {
            let file = archive
                .by_index_raw(index)
                .map_err(|e| invalid(e.to_string()))?;
            Ok((
                file.header_start() as usize,
                file.central_header_start() as usize,
                file.name() != PLATE_GCODE,
            ))
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    for (local, central, utf8) in records {
        if utf8 {
            bytes[local + 7] |= 0x08;
            bytes[central + 9] |= 0x08;
        }
        // Use the version needed to extract also as the DOS creator version.
        bytes.copy_within(central + 6..central + 8, central + 4);
        bytes[central + 38..central + 42].fill(0);
    }
    Ok(())
}
