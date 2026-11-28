use std::fmt::Write;

use super::DECL;
use crate::project::{
    export::{
        MODEL,
        saved_assets::{PREVIEW_REFERENCES, SMALL_PREVIEW},
    },
    relationships::MODEL_RELATIONSHIP_TYPE,
};

pub(in crate::project::export) fn root_relationships(has_previews: bool) -> String {
    let primary = PREVIEW_REFERENCES[0].1;
    let previews = [
        (
            2,
            primary,
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
        ),
        (
            4,
            primary,
            "http://schemas.bambulab.com/package/2021/cover-thumbnail-middle",
        ),
        (
            5,
            SMALL_PREVIEW,
            "http://schemas.bambulab.com/package/2021/cover-thumbnail-small",
        ),
    ];
    relationship(
        std::iter::once((1, MODEL, MODEL_RELATIONSHIP_TYPE))
            .chain(previews.into_iter().filter(|_| has_previews)),
    )
}

pub(in crate::project::export) fn relationships<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> String {
    typed_relationships(paths, MODEL_RELATIONSHIP_TYPE)
}

pub(in crate::project::export) fn gcode_relationship<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> String {
    typed_relationships(paths, "http://schemas.bambulab.com/package/2021/gcode")
}

fn typed_relationships<'a>(
    paths: impl IntoIterator<Item = &'a str>,
    media_type: &'static str,
) -> String {
    relationship(
        paths
            .into_iter()
            .enumerate()
            .map(|(index, path)| (index + 1, path, media_type)),
    )
}

fn relationship<'a>(entries: impl IntoIterator<Item = (usize, &'a str, &'static str)>) -> String {
    let mut out = format!(
        "{DECL}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n",
    );
    for (id, path, media_type) in entries {
        let path = quick_xml::escape::escape(path);
        writeln!(
            out,
            " <Relationship Target=\"/{path}\" Id=\"rel-{id}\" Type=\"{media_type}\"/>"
        )
        .unwrap();
    }
    out.push_str("</Relationships>");
    out
}
