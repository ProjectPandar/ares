//! The arranged build items and plate map: read from the Native export, written from the
//! session the way `_BBS_3MF_Exporter` writes them.

use std::io::{Cursor, Read};

use ares_core::PlaterSession;

use super::super::plater_archive::attribute;
use super::{MODEL, OBJECTS};

/// The arrange result as the exported project records it.
#[derive(Debug, PartialEq)]
pub(super) struct Arranged {
    /// `(object name, transform, printable)` per build item, in build order.
    items: Vec<(String, String, bool)>,
    /// `(object name, instance index)` per plate.
    plates: Vec<Vec<(String, usize)>>,
}

/// A transform value at the exporter's 9 significant digits.
fn digits(value: f64) -> String {
    format!("{value:.8e}")
}

fn read_entry(archive: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(archive)).unwrap();
    let mut text = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// Each element's text from `open` up to the next `close`.
fn elements<'a>(text: &'a str, open: &str, close: &str) -> Vec<&'a str> {
    text.match_indices(open)
        .map(|(start, _)| {
            let rest = &text[start..];
            &rest[..rest.find(close).unwrap()]
        })
        .collect()
}

/// The `value` of an element's first `<metadata key="{key}" .../>`.
fn metadata_value(element: &str, key: &str) -> String {
    let metadata = elements(element, &format!("key=\"{key}\""), "/>")[0];
    attribute(metadata, "value").to_owned()
}

impl Arranged {
    pub(super) fn from_native(project: &[u8]) -> Self {
        let objects = read_entry(project, OBJECTS);
        let name_of = |id: &str| {
            let object = elements(&objects, &format!("<object id=\"{id}\""), "</object>")[0];
            metadata_value(object, "name")
        };
        let items = elements(&read_entry(project, MODEL), "<item ", "/>")
            .into_iter()
            .map(|item| {
                let transform = attribute(item, "transform")
                    .split(' ')
                    .map(|value| digits(value.parse().unwrap()))
                    .collect::<Vec<_>>()
                    .join(" ");
                let printable = attribute(item, "printable") == "1";
                (name_of(attribute(item, "objectid")), transform, printable)
            })
            .collect();
        let plates = elements(&objects, "<plate>", "</plate>")
            .into_iter()
            .map(|plate| {
                elements(plate, "<model_instance>", "</model_instance>")
                    .into_iter()
                    .map(|instance| {
                        let index = metadata_value(instance, "instance_id").parse().unwrap();
                        (name_of(&metadata_value(instance, "object_id")), index)
                    })
                    .collect()
            })
            .collect();
        Self { items, plates }
    }

    /// The session's arrange as Native exports it: every plate, or with `only`, just that
    /// plate's instances while the other plates stay listed empty.
    pub(super) fn from_session(session: &PlaterSession, only: Option<usize>) -> Self {
        let objects = session.project().objects();
        let plates = session.plates();
        let exported =
            |key: &(usize, usize)| only.is_none_or(|plate| plates[plate].instances.contains(key));
        let items = objects
            .iter()
            .enumerate()
            .flat_map(|(object, item)| {
                (0..item.instances().len()).map(move |instance| (object, instance))
            })
            .filter(exported)
            .map(|(object, instance)| {
                let matrix = session
                    .instance_transform(object, instance)
                    .unwrap()
                    .to_column_major();
                let transform = (0..4)
                    .flat_map(|column| (0..3).map(move |row| matrix[column * 4 + row]))
                    .map(digits)
                    .collect::<Vec<_>>()
                    .join(" ");
                let placed = &objects[object].instances()[instance];
                (
                    objects[object].name().to_owned(),
                    transform,
                    placed.printable(),
                )
            })
            .collect();
        let plates = plates
            .iter()
            .map(|plate| {
                plate
                    .instances
                    .iter()
                    .filter(|key| exported(key))
                    .map(|&(object, instance)| (objects[object].name().to_owned(), instance))
                    .collect()
            })
            .collect();
        Self { items, plates }
    }
}
