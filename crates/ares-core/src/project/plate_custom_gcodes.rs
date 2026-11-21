//! The project's per-plate custom G-codes (`Model::plates_custom_gcodes`,
//! `Model.hpp:1562-1570`), loaded from `Metadata/custom_gcode_per_layer.xml`
//! (`bbs_3mf.cpp:220`, `:3208-3292`).
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    str::FromStr,
};

use quick_xml::events::{BytesStart, Event};

use crate::{
    SliceError,
    gcode_processor::{CustomGCodeItem, CustomGCodeType},
};

use super::{PackagePath, ProjectArchive, domain::Project, xml::configured_reader};

const CUSTOM_GCODE_PER_PRINT_Z_FILE: &str = "Metadata/custom_gcode_per_layer.xml";
const ROOT: &[u8] = b"custom_gcodes_per_layer";

/// `CustomGCode::Mode` (`CustomGCode.hpp:68-76`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CustomGCodeMode {
    #[default]
    Undef,
    SingleExtruder,
    MultiAsSingle,
    MultiExtruder,
}

/// `CustomGCode::Info` (`CustomGCode.hpp:82-92`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomGCodeInfo {
    pub mode: CustomGCodeMode,
    pub gcodes: Vec<CustomGCodeItem>,
}

/// A child of the root with its own children: the two ptree levels the
/// extractor reads.
struct Node<'a> {
    element: BytesStart<'a>,
    children: Vec<BytesStart<'a>>,
}

/// The `bbs_3mf.cpp:1948-1951` entry match, the extractor and the
/// `Model.cpp:365-366` mode check.
pub(crate) fn load(
    archive: &mut ProjectArchive<'_>,
    archive_paths: &BTreeSet<PackagePath>,
) -> Result<BTreeMap<i32, CustomGCodeInfo>, SliceError> {
    let mut paths = archive_paths.iter().filter(|path| {
        path.as_str()
            .eq_ignore_ascii_case(CUSTOM_GCODE_PER_PRINT_Z_FILE)
    });
    let Some(path) = paths.next().cloned() else {
        return Ok(BTreeMap::new());
    };
    if paths.next().is_some() {
        return Err(invalid(format_args!(
            "ambiguous archive path {CUSTOM_GCODE_PER_PRINT_Z_FILE}"
        )));
    }
    let mut plates = extract(&archive.read(&path)?)?;
    for info in plates.values_mut() {
        check_mode_for_custom_gcode_per_print_z(info);
    }
    Ok(plates)
}

/// `_extract_custom_gcode_per_print_z_from_archive` (`bbs_3mf.cpp:3208-3292`).
fn extract(input: &[u8]) -> Result<BTreeMap<i32, CustomGCodeInfo>, SliceError> {
    let mut plates = BTreeMap::new();
    if input.is_empty() {
        return Ok(plates);
    }
    let Some(root) = read_root(input)? else {
        return Ok(plates);
    };
    if root
        .iter()
        .any(|node| node.element.name().as_ref() == b"plate")
    {
        for node in root
            .iter()
            .filter(|node| node.element.name().as_ref() == b"plate")
        {
            // Every plate_info is read; the last one wins.
            let plate_id = node
                .children
                .iter()
                .filter(|code| code.name().as_ref() == b"plate_info")
                .try_fold(-1, |_, code| number::<i32>(code, "id"))?;
            if plate_id == -1 {
                continue;
            }
            extract_code(&mut plates, plate_id, node.children.iter())?;
        }
    } else {
        extract_code(&mut plates, 1, root.iter().map(|node| &node.element))?;
    }
    Ok(plates)
}

/// The root's children when the document's first node (`main_tree.front()`,
/// an element or a comment) is the `custom_gcodes_per_layer` element.
fn read_root(input: &[u8]) -> Result<Option<Vec<Node<'_>>>, SliceError> {
    let mut reader = configured_reader(input);
    let mut depth = 0usize;
    let mut front_seen = false;
    let mut collecting = false;
    let mut root = Vec::new();
    loop {
        let (element, empty) = match reader.read_event().map_err(invalid)? {
            Event::Start(element) => (element, false),
            Event::Empty(element) => (element, true),
            Event::End(_) => {
                depth -= 1;
                continue;
            }
            Event::Comment(_) if depth == 0 => {
                front_seen = true;
                collecting = false;
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        match depth {
            0 => {
                collecting = !front_seen && element.name().as_ref() == ROOT;
                front_seen = true;
            }
            1 if collecting => root.push(Node {
                element,
                children: Vec::new(),
            }),
            2 if collecting => {
                if let Some(node) = root.last_mut() {
                    node.children.push(element);
                }
            }
            _ => {}
        }
        if !empty {
            depth += 1;
        }
    }
    if !front_seen || depth != 0 {
        return Err(invalid("missing or unclosed root element"));
    }
    Ok(collecting.then_some(root))
}

/// The `extract_code` lambda (`bbs_3mf.cpp:3226-3263`).
fn extract_code<'a, 'b: 'a>(
    plates: &mut BTreeMap<i32, CustomGCodeInfo>,
    plate_id: i32,
    codes: impl Iterator<Item = &'a BytesStart<'b>>,
) -> Result<(), SliceError> {
    let key = plate_id
        .checked_sub(1)
        .ok_or_else(|| invalid(format_args!("plate id {plate_id} is out of range")))?;
    for code in codes {
        match code.name().as_ref() {
            b"mode" => {
                let mode = required(code, "value")?;
                plates.entry(key).or_default().mode = match mode.as_str() {
                    "SingleExtruder" => CustomGCodeMode::SingleExtruder,
                    "MultiAsSingle" => CustomGCodeMode::MultiAsSingle,
                    _ => CustomGCodeMode::MultiExtruder,
                };
            }
            b"layer" => {
                let print_z = double(code, "top_z")?;
                let extruder = number::<i32>(code, "extruder")?;
                let color = required(code, "color")?;
                let (kind, extra) = if attribute(code, "type")?.is_none() {
                    // PrusaSlicer 2.2.0 and older.
                    let gcode = required(code, "gcode")?;
                    let kind = match gcode.as_str() {
                        "M600" => CustomGCodeType::ColorChange,
                        "M601" => CustomGCodeType::PausePrint,
                        "tool_change" => CustomGCodeType::ToolChange,
                        _ => CustomGCodeType::Custom,
                    };
                    let extra = match kind {
                        CustomGCodeType::PausePrint => color.clone(),
                        CustomGCodeType::Custom => gcode,
                        _ => String::new(),
                    };
                    (kind, extra)
                } else {
                    (
                        custom_gcode_type(number(code, "type")?)?,
                        required(code, "extra")?,
                    )
                };
                let extruder = u32::try_from(extruder)
                    .map_err(|_| invalid(format_args!("extruder {extruder} is out of range")))?;
                plates.entry(key).or_default().gcodes.push(CustomGCodeItem {
                    print_z,
                    kind,
                    extruder,
                    color,
                    extra,
                });
            }
            _ => {}
        }
    }
    Ok(())
}

/// `CustomGCode::Type` from its integer (`CustomGCode.hpp:14-22`).
fn custom_gcode_type(value: i32) -> Result<CustomGCodeType, SliceError> {
    Ok(match value {
        0 => CustomGCodeType::ColorChange,
        1 => CustomGCodeType::PausePrint,
        2 => CustomGCodeType::ToolChange,
        3 => CustomGCodeType::Template,
        4 => CustomGCodeType::Custom,
        5 => CustomGCodeType::Unknown,
        _ => return Err(invalid(format_args!("type {value} is out of range"))),
    })
}

/// `check_mode_for_custom_gcode_per_print_z` (`CustomGCode.cpp:41-58`).
fn check_mode_for_custom_gcode_per_print_z(info: &mut CustomGCodeInfo) {
    if info.mode != CustomGCodeMode::Undef {
        return;
    }
    let mut is_single_extruder = true;
    for item in &info.gcodes {
        if item.kind == CustomGCodeType::ToolChange {
            info.mode = CustomGCodeMode::MultiAsSingle;
            return;
        }
        if item.kind == CustomGCodeType::ColorChange && item.extruder > 1 {
            is_single_extruder = false;
        }
    }
    info.mode = if is_single_extruder {
        CustomGCodeMode::SingleExtruder
    } else {
        CustomGCodeMode::MultiExtruder
    };
}

/// The first attribute named `key`, entity-decoded without whitespace
/// normalization (rapidxml under `pt::read_xml`).
fn attribute(element: &BytesStart<'_>, key: &str) -> Result<Option<String>, SliceError> {
    let mut attributes = element.attributes();
    attributes.with_checks(false);
    for attribute in attributes {
        let attribute = attribute.map_err(invalid)?;
        if attribute.key.as_ref() == key.as_bytes() {
            let raw = std::str::from_utf8(&attribute.value).map_err(invalid)?;
            return Ok(Some(
                quick_xml::escape::unescape(raw)
                    .map_err(invalid)?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

/// `ptree::get<std::string>("<xmlattr>.key")`.
fn required(element: &BytesStart<'_>, key: &str) -> Result<String, SliceError> {
    attribute(element, key)?.ok_or_else(|| invalid(format_args!("No such node (<xmlattr>.{key})")))
}

/// `ptree::get<T>`: stream extraction that skips surrounding whitespace and
/// must consume the whole value.
fn number<T: FromStr>(element: &BytesStart<'_>, key: &str) -> Result<T, SliceError> {
    let value = required(element, key)?;
    value
        .trim_matches([' ', '\t', '\n', '\x0B', '\x0C', '\r'])
        .parse()
        .map_err(|_| {
            invalid(format_args!(
                "conversion of <xmlattr>.{key} {value:?} failed"
            ))
        })
}

/// `ptree::get<double>`: the stream also fails on `inf`, `nan` and overflow.
fn double(element: &BytesStart<'_>, key: &str) -> Result<f64, SliceError> {
    let value = number::<f64>(element, key)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(invalid(format_args!(
            "conversion of <xmlattr>.{key} failed"
        )))
    }
}

fn invalid(reason: impl Display) -> SliceError {
    SliceError::InvalidInput(format!(
        "invalid project custom G-codes per layer XML: {reason}"
    ))
}

impl Project {
    /// `CLI::run` (`OrcaSlicer.cpp:4117–4134`) removes only Custom items.
    pub(crate) fn remove_custom_plate_gcodes(&mut self) {
        for info in self.plates_custom_gcodes.values_mut() {
            info.gcodes
                .retain(|item| item.kind != CustomGCodeType::Custom);
        }
    }

    /// `Model::plates_custom_gcodes`, keyed by the zero-based plate index.
    pub fn plates_custom_gcodes(&self) -> &BTreeMap<i32, CustomGCodeInfo> {
        &self.plates_custom_gcodes
    }

    /// `Model::get_curr_plate_custom_gcodes` (`Model.hpp:1565-1570`): the
    /// selected plate's entry, or an empty `Info`. An unselected project has
    /// `curr_plate_index` 0, as upstream right after loading.
    pub fn curr_plate_custom_gcodes(&self) -> CustomGCodeInfo {
        self.plates_custom_gcodes
            .get(&self.curr_plate_index)
            .cloned()
            .unwrap_or_default()
    }
}
