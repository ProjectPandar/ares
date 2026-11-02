/// Rebuilds the 3MF with `Metadata/project_settings.config` replaced by
/// `settings`; every other member is copied verbatim.
fn rewrite_project_settings(archive_bytes: &[u8], settings: &[u8]) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(std::io::Cursor::new(archive_bytes)).unwrap();
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_owned();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut bytes).unwrap();
        writer.start_file(&name, options).unwrap();
        if name.eq_ignore_ascii_case("Metadata/project_settings.config") {
            std::io::Write::write_all(&mut writer, settings).unwrap();
        } else {
            std::io::Write::write_all(&mut writer, &bytes).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
