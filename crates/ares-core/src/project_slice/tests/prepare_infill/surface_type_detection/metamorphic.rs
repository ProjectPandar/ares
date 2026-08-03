use std::io::{Cursor, Read, Write};

use zip::{CompressionMethod, System, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::project_slice::prepare_infill::surface_type_detection::PreparedPostSurfaceTypeDetection;

use super::{super::super::support::KsrArchive, fixture::prepare, ksr::checksum};

#[test]
fn task22o17_zip_repack_and_non_slicing_name_do_not_select_fixture_behavior() {
    let baseline = prepare(KsrArchive::new().bytes());
    let repacked = prepare(repack_reverse(KsrArchive::new().bytes()));
    assert_eq!(checksum(&baseline), checksum(&repacked));

    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "ksr_fdmtest_v4.drc",
        "semantically_identical_name.drc",
    );
    let renamed = prepare(renamed.bytes());
    assert_eq!(checksum(&baseline), checksum(&renamed));
}

#[test]
fn task22o17_exact_geometry_scaling_changes_the_source_derived_span() {
    let baseline = prepare(KsrArchive::new().bytes());
    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "1 0 0 0 1 0 0 0 1 0 0 0",
        "2 0 0 0 1 0 0 0 1 0 0 0",
    );
    let scaled = prepare(scaled.bytes());
    assert_ne!(checksum(&baseline), checksum(&scaled));
    assert_eq!(baseline.objects.len(), scaled.objects.len());
    assert_eq!(
        first_contour_x_span(&scaled),
        first_contour_x_span(&baseline) * 2 + 300_000
    );
}

fn first_contour_x_span(prepared: &PreparedPostSurfaceTypeDetection) -> i64 {
    let points = prepared.objects[0].records[0].as_ref().unwrap().slices[0]
        .as_parts()
        .1
        .contour()
        .points();
    let minimum = points.iter().map(|point| point.x()).min().unwrap();
    let maximum = points.iter().map(|point| point.x()).max().unwrap();
    maximum - minimum
}

fn repack_reverse(bytes: Vec<u8>) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        if file.is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        entries.push((file.name().to_owned(), bytes));
    }
    entries.reverse();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .system(System::Unix);
    for (path, bytes) in entries {
        writer.start_file(path, options).unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
