//! 3MF text helpers the plater parity tests share: rewriting archive entries and reading XML
//! attributes.

use std::io::{Cursor, Read, Write};

pub(super) fn attribute<'a>(element: &'a str, name: &str) -> &'a str {
    let start = element.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
    &element[start..start + element[start..].find('"').unwrap()]
}

/// `input` with the text entries `edit` returns replaced.
pub(super) fn rewrite_archive(
    input: &[u8],
    edit: impl Fn(&str, &str) -> Option<String>,
) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        let bytes = match std::str::from_utf8(&bytes)
            .ok()
            .and_then(|text| edit(file.name(), text))
        {
            Some(text) => text.into_bytes(),
            None => bytes,
        };
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}
