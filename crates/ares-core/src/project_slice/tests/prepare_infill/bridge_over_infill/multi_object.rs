use crate::{
    ProcessInfillPattern, ProcessInternalBridgeFilter,
    project_slice::{
        prepare_infill::{bridge_over_infill, external_surfaces},
        region_slices::RegionSurfaceKind,
        tests::support::KsrArchive,
    },
};

const FIRST_LEAF: &str = "3D/Objects/ksr_fdmtest_v4.drc_2.model";
const SECOND_LEAF: &str = "3D/Objects/ksr_fdmtest_v4.drc_3.model";

#[test]
fn task22o43_two_source_objects_keep_filter_and_candidate_ownership() {
    let horizontal =
        super::super::horizontal_shell_propagation::fixture::prepare(two_object_archive().bytes());
    let mut external = external_surfaces::prepare(horizontal).unwrap();
    {
        let traversal = &mut external.predecessor.predecessor;
        assert_eq!(
            traversal
                .objects
                .iter()
                .map(source_identity)
                .collect::<Vec<_>>(),
            [(0, 0), (1, 0)]
        );
        assert_eq!(
            traversal
                .resolved
                .objects
                .iter()
                .map(|object| object.source_object_index)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        traversal.resolved.objects[0]
            .object
            .dont_filter_internal_bridges = ProcessInternalBridgeFilter::Disabled;
        traversal.resolved.objects[1]
            .object
            .dont_filter_internal_bridges = ProcessInternalBridgeFilter::Limited;
        assert_eq!(
            traversal
                .resolved
                .views
                .full
                .process
                .region
                .sparse_infill_pattern,
            ProcessInfillPattern::CrossHatch
        );
        let second_source_prelude = &mut traversal.objects[1]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        second_source_prelude.object.object.as_parts_mut().0.regions[0]
            .options
            .sparse_infill_pattern = ProcessInfillPattern::Lightning;
        assert_eq!(
            traversal
                .objects
                .iter()
                .map(retained_pattern)
                .collect::<Vec<_>>(),
            [
                ProcessInfillPattern::CrossHatch,
                ProcessInfillPattern::Lightning,
            ]
        );
    }

    let prepared = bridge_over_infill::prepare(external).unwrap();
    assert_eq!(prepared.objects.len(), 2);
    assert_eq!(
        prepared
            .objects
            .iter()
            .map(|object| object.has_lightning_infill)
            .collect::<Vec<_>>(),
        [false, true]
    );
    assert_eq!(
        prepared
            .objects
            .iter()
            .map(|object| {
                (
                    object.surfaces_by_layer.len(),
                    object.surfaces_by_layer.values().map(Vec::len).sum(),
                    object
                        .surfaces_by_layer
                        .values()
                        .flatten()
                        .map(|candidate| candidate.new_polygons.len())
                        .sum(),
                )
            })
            .collect::<Vec<_>>(),
        [(18, 43, 53), (58, 100, 166)]
    );

    let horizontal = &prepared.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    assert_eq!(
        traversal
            .resolved
            .views
            .full
            .process
            .region
            .sparse_infill_pattern,
        ProcessInfillPattern::CrossHatch
    );
    let identities = [(0, 0), (1, 0)];
    for (object_index, candidates) in prepared.objects.iter().enumerate() {
        let prelude = &traversal.objects[object_index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        assert_eq!(prelude.object.identity(), identities[object_index]);
        let (_, inputs) = prelude.object.as_parts();
        for (&layer_index, surfaces) in &candidates.surfaces_by_layer {
            let input = inputs[layer_index].as_ref().unwrap();
            assert_eq!(input.source_object_index, identities[object_index].0);
            assert_eq!(input.transform_index, identities[object_index].1);
            for candidate in surfaces {
                assert_eq!(candidate.source.layer_index, layer_index);
                assert_eq!(candidate.source.region_index, input.current.region_index);
                assert_eq!(
                    horizontal.objects[object_index].records[layer_index]
                        .as_ref()
                        .unwrap()
                        .fill_surfaces[candidate.source.surface_index]
                        .as_parts()
                        .0,
                    RegionSurfaceKind::InternalSolid
                );
            }
        }
    }

    bridge_over_infill::dispose(prepared);
}

fn source_identity(
    object: &crate::project_slice::perimeters::classic::traversal::PostClassicTraversalPrintObject,
) -> (usize, usize) {
    object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity()
}

fn retained_pattern(
    object: &crate::project_slice::perimeters::classic::traversal::PostClassicTraversalPrintObject,
) -> ProcessInfillPattern {
    object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .object
        .as_parts()
        .0
        .regions[0]
        .options
        .sparse_infill_pattern
}

pub(in crate::project_slice) fn two_object_archive() -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.copy_entry(FIRST_LEAF, SECOND_LEAF);
    archive.replace_unique(
        SECOND_LEAF,
        r#"<object id="1" p:UUID="00020000-81cb-4c03-9d28-80fed5dfa1dc""#,
        r#"<object id="4" p:UUID="00040000-81cb-4c03-9d28-80fed5dfa1dc""#,
    );
    archive.replace(
        "3D/_rels/3dmodel.model.rels",
        "</Relationships>",
        concat!(
            " <Relationship Target=\"/3D/Objects/ksr_fdmtest_v4.drc_3.model\" ",
            "Id=\"rel-2\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n",
            "</Relationships>",
        ),
    );
    archive.replace(
        "3D/3dmodel.model",
        " </resources>",
        concat!(
            "  <object id=\"3\" type=\"model\">\n",
            "   <components>\n",
            "    <component p:path=\"/3D/Objects/ksr_fdmtest_v4.drc_3.model\" ",
            "objectid=\"4\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"/>\n",
            "   </components>\n",
            "  </object>\n",
            " </resources>",
        ),
    );
    archive.replace(
        "3D/3dmodel.model",
        " </build>",
        concat!(
            "  <item objectid=\"3\" transform=\"1 0 0 0 1 0 0 0 1 133.039205 115.992105 46\" ",
            "printable=\"1\" auto_drop=\"1\"/>\n",
            " </build>",
        ),
    );
    archive.replace(
        "Metadata/model_settings.config",
        "  <plate>",
        concat!(
            "  <object id=\"3\">\n",
            "    <metadata key=\"name\" value=\"ksr_fdmtest_v4-copy.drc\"/>\n",
            "    <metadata key=\"extruder\" value=\"1\"/>\n",
            "    <part id=\"4\" subtype=\"normal_part\">\n",
            "      <metadata key=\"name\" value=\"ksr_fdmtest_v4-copy.drc\"/>\n",
            "      <metadata key=\"matrix\" value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"/>\n",
            "      <metadata key=\"source_file\" value=\"ksr_fdmtest_v4.drc\"/>\n",
            "      <metadata key=\"source_object_id\" value=\"0\"/>\n",
            "      <metadata key=\"source_volume_id\" value=\"0\"/>\n",
            "      <metadata key=\"source_offset_x\" value=\"128.5\"/>\n",
            "      <metadata key=\"source_offset_y\" value=\"128.5\"/>\n",
            "      <metadata key=\"source_offset_z\" value=\"46\"/>\n",
            "    </part>\n",
            "  </object>\n",
            "  <plate>",
        ),
    );
    archive.replace(
        "Metadata/model_settings.config",
        "  </plate>",
        concat!(
            "    <model_instance>\n",
            "      <metadata key=\"object_id\" value=\"3\"/>\n",
            "      <metadata key=\"instance_id\" value=\"0\"/>\n",
            "      <metadata key=\"identify_id\" value=\"134\"/>\n",
            "    </model_instance>\n",
            "  </plate>",
        ),
    );
    archive.replace(
        "Metadata/model_settings.config",
        "  </assemble>",
        concat!(
            "   <assemble_item object_id=\"3\" instance_id=\"0\" ",
            "transform=\"1 0 0 0 1 0 0 0 1 0 0 46\" offset=\"0 0 0\" />\n",
            "  </assemble>",
        ),
    );
    archive
}
