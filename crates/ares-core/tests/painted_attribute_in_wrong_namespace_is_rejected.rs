use std::io::{Cursor, Write};

use ares_core::{GenerationMetadata, SliceError, SlicerContext, load_project, slice_project};
use zip::{ZipWriter, write::SimpleFileOptions};

#[tokio::test]
async fn painted_attribute_in_wrong_namespace_is_rejected() {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    archive.write_all(br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="png" ContentType="image/png"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>"#).unwrap();
    archive
        .start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    archive.write_all(br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/3dmodel.model" Id="r1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#).unwrap();
    archive
        .start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();
    archive.write_all(br#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:evil="https://example.invalid/production" unit="millimeter"><resources><object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2" evil:paint_fuzzy_skin="4"/></triangles></mesh></object></resources><build><item objectid="1"/></build></model>"#).unwrap();
    let bytes = archive.finish().unwrap().into_inner();
    let load_error = load_project(&bytes).unwrap_err();
    let slice_error = slice_project(
        &bytes,
        GenerationMetadata::deterministic(2026, 7, 19, 1, 2, 3),
        SlicerContext::new(0),
    )
    .await
    .unwrap_err();
    assert_eq!(
        (load_error, slice_error),
        (
            SliceError::InvalidInput(
                "invalid project model XML: attribute namespace does not match its 3MF meaning"
                    .to_owned()
            ),
            SliceError::InvalidInput(
                "invalid project model XML: attribute namespace does not match its 3MF meaning"
                    .to_owned()
            ),
        )
    );
}
