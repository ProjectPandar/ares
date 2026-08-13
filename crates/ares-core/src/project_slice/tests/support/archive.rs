use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
};

use zip::{CompressionMethod, DateTime, System, ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::KSR_PROJECT;

const FLUSH_MATRIX: &str = concat!(
    "\t\"flush_volumes_matrix\": [\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"0\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"280\",\r\n",
    "\t\t\"0\"\r\n",
    "\t]",
);
const INVALID_FLUSH_MATRIX: &str = "\t\"flush_volumes_matrix\": [\r\n\t\t\"0\"\r\n\t]";

#[derive(Clone)]
pub(in crate::project_slice::tests) struct KsrArchive {
    entries: BTreeMap<String, Vec<u8>>,
}

impl KsrArchive {
    pub(in crate::project_slice::tests) fn new() -> Self {
        let mut archive = ZipArchive::new(Cursor::new(KSR_PROJECT)).unwrap();
        let mut entries = BTreeMap::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            if file.is_dir() {
                continue;
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            entries.insert(file.name().to_owned(), bytes);
        }
        Self { entries }
    }

    pub(in crate::project_slice::tests) fn insert_text(&mut self, path: &str, text: &str) {
        self.entries
            .insert(path.to_owned(), text.as_bytes().to_vec());
    }

    pub(in crate::project_slice::tests) fn copy_entry(&mut self, from: &str, to: &str) {
        let bytes = self.entries.get(from).unwrap().clone();
        assert!(self.entries.insert(to.to_owned(), bytes).is_none());
    }

    pub(in crate::project_slice::tests) fn replace(&mut self, path: &str, from: &str, to: &str) {
        let text = String::from_utf8(self.entries.remove(path).unwrap()).unwrap();
        assert!(text.contains(from), "{path} does not contain {from:?}");
        self.entries
            .insert(path.to_owned(), text.replace(from, to).into_bytes());
    }

    pub(in crate::project_slice::tests) fn replace_unique(
        &mut self,
        path: &str,
        from: &str,
        to: &str,
    ) {
        let text = String::from_utf8(self.entries.remove(path).unwrap()).unwrap();
        assert_eq!(
            text.match_indices(from).count(),
            1,
            "{path} must contain exactly one {from:?}"
        );
        let replaced = text.replacen(from, to, 1);
        assert_eq!(replaced.match_indices(from).count(), 0);
        assert_eq!(replaced.match_indices(to).count(), 1);
        self.entries.insert(path.to_owned(), replaced.into_bytes());
    }

    pub(in crate::project_slice::tests) fn invalidate_flush_matrix(&mut self) {
        self.replace(
            "Metadata/project_settings.config",
            FLUSH_MATRIX,
            INVALID_FLUSH_MATRIX,
        );
    }

    pub(in crate::project_slice::tests) fn repair_flush_matrix(&mut self) {
        self.replace(
            "Metadata/project_settings.config",
            INVALID_FLUSH_MATRIX,
            FLUSH_MATRIX,
        );
    }

    pub(in crate::project_slice::tests) fn bytes(self) -> Vec<u8> {
        self.write(CompressionMethod::Deflated, System::Dos, false, None)
    }

    pub(in crate::project_slice::tests) fn bytes_stored_reverse(self) -> Vec<u8> {
        self.write(CompressionMethod::Stored, System::Unix, true, None)
    }

    pub(in crate::project_slice::tests) fn bytes_with_timestamp(self) -> Vec<u8> {
        self.write(
            CompressionMethod::Deflated,
            System::Dos,
            false,
            Some(DateTime::from_date_and_time(2037, 11, 19, 20, 21, 22).unwrap()),
        )
    }

    fn write(
        self,
        compression: CompressionMethod,
        system: System,
        reverse: bool,
        timestamp: Option<DateTime>,
    ) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let mut options = SimpleFileOptions::default()
            .compression_method(compression)
            .system(system);
        if let Some(timestamp) = timestamp {
            options = options.last_modified_time(timestamp);
        }
        let mut entries = self.entries.into_iter().collect::<Vec<_>>();
        if reverse {
            entries.reverse();
        }
        for (path, bytes) in entries {
            writer.start_file(path, options).unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}
