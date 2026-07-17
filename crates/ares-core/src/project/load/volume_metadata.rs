use crate::{
    SliceError,
    options::RegionOptionOverrides,
    project::{
        ProjectVolumeType, Transform3d,
        model_settings::{Metadata, PartSettings},
    },
};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SelectedSourceProvenance {
    pub(crate) input_file: String,
    pub(crate) object_index: i32,
    pub(crate) volume_index: i32,
    pub(crate) offset: [f64; 3],
    pub(crate) converted_from_inches: bool,
    pub(crate) converted_from_meters: bool,
    pub(crate) from_builtin_objects: bool,
}

impl Default for SelectedSourceProvenance {
    fn default() -> Self {
        Self {
            input_file: String::new(),
            object_index: -1,
            volume_index: -1,
            offset: [0.0; 3],
            converted_from_inches: false,
            converted_from_meters: false,
            from_builtin_objects: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SelectedMeshStatistics {
    edges_fixed: u32,
    degenerate_facets: u32,
    facets_removed: u32,
    facets_reversed: u32,
    backwards_edges: u32,
}

impl SelectedMeshStatistics {
    #[cfg(test)]
    pub(crate) fn as_array(self) -> [u32; 5] {
        [
            self.edges_fixed,
            self.degenerate_facets,
            self.facets_removed,
            self.facets_reversed,
            self.backwards_edges,
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct SelectedVolumeMetadata {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) volume_type: ProjectVolumeType,
    pub(crate) region_overrides: RegionOptionOverrides,
    pub(crate) source_transform: Transform3d,
    pub(crate) mesh_shared: bool,
    pub(crate) source_provenance: SelectedSourceProvenance,
    pub(crate) mesh_statistics: SelectedMeshStatistics,
}

pub(super) fn select(
    parts: &[PartSettings],
    leaf_index: usize,
    leaf_id: u32,
) -> Result<SelectedVolumeMetadata, SliceError> {
    let selected = parts
        .get(leaf_index)
        .filter(|part| part.id == leaf_id)
        .or_else(|| parts.iter().find(|part| part.id == leaf_id));
    selected.map_or_else(
        || {
            Ok(SelectedVolumeMetadata {
                id: leaf_id,
                name: String::new(),
                volume_type: ProjectVolumeType::ModelPart,
                region_overrides: RegionOptionOverrides::default(),
                source_transform: Transform3d::IDENTITY,
                mesh_shared: false,
                source_provenance: SelectedSourceProvenance::default(),
                mesh_statistics: SelectedMeshStatistics::default(),
            })
        },
        from_part,
    )
}

pub(super) fn validate(part: &PartSettings) -> Result<(), SliceError> {
    from_part(part).map(|_| ())
}

#[cfg(test)]
pub(crate) fn selected_volume_metadata_for_test(
    parts: &[PartSettings],
    leaf_index: usize,
    leaf_id: u32,
) -> Result<SelectedVolumeMetadata, SliceError> {
    select(parts, leaf_index, leaf_id)
}

fn from_part(part: &PartSettings) -> Result<SelectedVolumeMetadata, SliceError> {
    let mut name = String::new();
    let mut volume_type = parse_volume_type("subtype", &part.subtype)?;
    let mut source_transform = Transform3d::IDENTITY;
    let mut has_source_transform = false;
    let mut mesh_shared = false;
    let mut source_provenance = SelectedSourceProvenance::default();

    for entry in &part.retained_metadata {
        match entry.key.as_str() {
            "name" => name.clone_from(&entry.value),
            "volume_type" | "part_type" => {
                volume_type = parse_volume_type(&entry.key, &entry.value)?;
            }
            "matrix" => {
                if has_source_transform {
                    return Err(invalid("repeated metadata \"matrix\""));
                }
                source_transform = Transform3d::parse_row_major(&entry.value)
                    .map_err(|_| invalid("matrix is not a valid row-major transform"))?;
                has_source_transform = true;
            }
            "source_file" => source_provenance.input_file.clone_from(&entry.value),
            "source_object_id" => {
                source_provenance.object_index = parse_i32(entry)?;
            }
            "source_volume_id" => {
                source_provenance.volume_index = parse_i32(entry)?;
            }
            "source_offset_x" => source_provenance.offset[0] = parse_f64(entry)?,
            "source_offset_y" => source_provenance.offset[1] = parse_f64(entry)?,
            "source_offset_z" => source_provenance.offset[2] = parse_f64(entry)?,
            "source_in_inches" => {
                source_provenance.converted_from_inches = entry.value == "1";
            }
            "source_in_meters" => {
                source_provenance.converted_from_meters = entry.value == "1";
            }
            "mesh_shared" => mesh_shared = true,
            _ => unreachable!("part structural metadata was classified before assembly"),
        }
    }

    let mesh_statistics =
        part.mesh_stat
            .as_ref()
            .map_or_else(SelectedMeshStatistics::default, |statistics| {
                SelectedMeshStatistics {
                    edges_fixed: statistics.edges_fixed,
                    degenerate_facets: statistics.degenerate_facets,
                    facets_removed: statistics.facets_removed,
                    facets_reversed: statistics.facets_reversed,
                    backwards_edges: statistics.backwards_edges,
                }
            });
    Ok(SelectedVolumeMetadata {
        id: part.id,
        name,
        volume_type,
        region_overrides: part.region_overrides.clone(),
        source_transform,
        mesh_shared,
        source_provenance,
        mesh_statistics,
    })
}

fn parse_volume_type(key: &str, value: &str) -> Result<ProjectVolumeType, SliceError> {
    match value {
        "normal_part" => Ok(ProjectVolumeType::ModelPart),
        "negative_part" => Ok(ProjectVolumeType::NegativeVolume),
        "modifier_part" => Ok(ProjectVolumeType::ParameterModifier),
        "support_enforcer" => Ok(ProjectVolumeType::SupportEnforcer),
        "support_blocker" => Ok(ProjectVolumeType::SupportBlocker),
        _ => Err(invalid(format!(
            "{key} \"{}\" is not a fixed volume type",
            bounded(value)
        ))),
    }
}

fn parse_i32(entry: &Metadata) -> Result<i32, SliceError> {
    entry.value.parse().map_err(|_| {
        invalid(format!(
            "{} \"{}\" is not an integer",
            entry.key,
            bounded(&entry.value)
        ))
    })
}

fn parse_f64(entry: &Metadata) -> Result<f64, SliceError> {
    let value = entry.value.parse::<f64>().map_err(|_| {
        invalid(format!(
            "{} \"{}\" is not a number",
            entry.key,
            bounded(&entry.value)
        ))
    })?;
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| invalid(format!("{} must be finite", entry.key)))
}

fn bounded(value: &str) -> String {
    let mut escaped = value.escape_debug();
    let output = escaped.by_ref().take(96).collect::<String>();
    if escaped.next().is_some() {
        let mut output = output;
        output.push_str("...");
        output
    } else {
        output
    }
}

fn invalid(reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project volume metadata: {reason}"))
}
