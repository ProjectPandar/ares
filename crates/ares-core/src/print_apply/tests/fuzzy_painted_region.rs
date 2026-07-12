use super::super::fuzzy_painted_region_state::{
    StagedFuzzyPaintedRegionConfigDerivation, StagedFuzzyPaintedRegionInput,
    StagedFuzzyPaintedRegionParent, StagedFuzzySkinConfig, StagedFuzzySkinType,
    staged_fuzzy_painted_region_configs,
};

fn config(region_id: usize, marker: u64, fuzzy_skin: StagedFuzzySkinType) -> StagedFuzzySkinConfig {
    StagedFuzzySkinConfig::new(region_id, marker, fuzzy_skin)
}

fn fuzzy_region(
    fuzzy_region_id: usize,
    parent: StagedFuzzyPaintedRegionParent,
    destination_region_id: usize,
) -> StagedFuzzyPaintedRegionInput {
    StagedFuzzyPaintedRegionInput::new(fuzzy_region_id, parent, destination_region_id)
}

fn derivation(
    fuzzy_region_id: usize,
    parent: StagedFuzzyPaintedRegionParent,
    destination_region_id: usize,
    config: StagedFuzzySkinConfig,
) -> StagedFuzzyPaintedRegionConfigDerivation {
    StagedFuzzyPaintedRegionConfigDerivation::new(
        fuzzy_region_id,
        parent,
        destination_region_id,
        config,
    )
}

#[test]
fn fuzzy_painted_region_config_derives_from_volume_region_parent() {
    let volume_parents = [config(10, 99, StagedFuzzySkinType::External)];
    let painted_parents = [];

    let derived = staged_fuzzy_painted_region_configs(
        &volume_parents,
        &painted_parents,
        &[fuzzy_region(
            4,
            StagedFuzzyPaintedRegionParent::VolumeRegion(0),
            40,
        )],
    );

    assert_eq!(
        derived,
        [derivation(
            4,
            StagedFuzzyPaintedRegionParent::VolumeRegion(0),
            40,
            config(10, 99, StagedFuzzySkinType::All),
        )]
    );
}

#[test]
fn fuzzy_painted_region_config_derives_from_painted_region_parent() {
    let volume_parents = [];
    let painted_parents = [config(20, 88, StagedFuzzySkinType::Hole)];

    let derived = staged_fuzzy_painted_region_configs(
        &volume_parents,
        &painted_parents,
        &[fuzzy_region(
            5,
            StagedFuzzyPaintedRegionParent::PaintedRegion(0),
            50,
        )],
    );

    assert_eq!(
        derived,
        [derivation(
            5,
            StagedFuzzyPaintedRegionParent::PaintedRegion(0),
            50,
            config(20, 88, StagedFuzzySkinType::All),
        )]
    );
}

#[test]
fn fuzzy_painted_region_config_normalizes_non_disabled_values_to_all() {
    let volume_parents = [
        config(1, 10, StagedFuzzySkinType::None),
        config(2, 20, StagedFuzzySkinType::External),
        config(3, 30, StagedFuzzySkinType::Hole),
        config(4, 40, StagedFuzzySkinType::AllWalls),
    ];
    let painted_parents = [];
    let fuzzy_regions = [
        fuzzy_region(1, StagedFuzzyPaintedRegionParent::VolumeRegion(0), 101),
        fuzzy_region(2, StagedFuzzyPaintedRegionParent::VolumeRegion(1), 102),
        fuzzy_region(3, StagedFuzzyPaintedRegionParent::VolumeRegion(2), 103),
        fuzzy_region(4, StagedFuzzyPaintedRegionParent::VolumeRegion(3), 104),
    ];

    let derived =
        staged_fuzzy_painted_region_configs(&volume_parents, &painted_parents, &fuzzy_regions);

    assert_eq!(
        derived,
        [
            derivation(
                1,
                StagedFuzzyPaintedRegionParent::VolumeRegion(0),
                101,
                config(1, 10, StagedFuzzySkinType::All)
            ),
            derivation(
                2,
                StagedFuzzyPaintedRegionParent::VolumeRegion(1),
                102,
                config(2, 20, StagedFuzzySkinType::All)
            ),
            derivation(
                3,
                StagedFuzzyPaintedRegionParent::VolumeRegion(2),
                103,
                config(3, 30, StagedFuzzySkinType::All)
            ),
            derivation(
                4,
                StagedFuzzyPaintedRegionParent::VolumeRegion(3),
                104,
                config(4, 40, StagedFuzzySkinType::All)
            ),
        ]
    );
}

#[test]
fn fuzzy_painted_region_config_preserves_disabled_fuzzy_skin() {
    let volume_parents = [config(9, 90, StagedFuzzySkinType::DisabledFuzzy)];
    let painted_parents = [];

    let derived = staged_fuzzy_painted_region_configs(
        &volume_parents,
        &painted_parents,
        &[fuzzy_region(
            9,
            StagedFuzzyPaintedRegionParent::VolumeRegion(0),
            109,
        )],
    );

    assert_eq!(
        derived,
        [derivation(
            9,
            StagedFuzzyPaintedRegionParent::VolumeRegion(0),
            109,
            config(9, 90, StagedFuzzySkinType::DisabledFuzzy),
        )]
    );
}

#[test]
fn fuzzy_painted_region_config_derives_multiple_regions_in_source_order() {
    let volume_parents = [config(1, 10, StagedFuzzySkinType::External)];
    let painted_parents = [config(2, 20, StagedFuzzySkinType::DisabledFuzzy)];

    let derived = staged_fuzzy_painted_region_configs(
        &volume_parents,
        &painted_parents,
        &[
            fuzzy_region(7, StagedFuzzyPaintedRegionParent::PaintedRegion(0), 207),
            fuzzy_region(6, StagedFuzzyPaintedRegionParent::VolumeRegion(0), 106),
        ],
    );

    assert_eq!(
        derived,
        [
            derivation(
                7,
                StagedFuzzyPaintedRegionParent::PaintedRegion(0),
                207,
                config(2, 20, StagedFuzzySkinType::DisabledFuzzy),
            ),
            derivation(
                6,
                StagedFuzzyPaintedRegionParent::VolumeRegion(0),
                106,
                config(1, 10, StagedFuzzySkinType::All),
            ),
        ]
    );
}

#[test]
fn fuzzy_painted_region_config_does_not_mutate_parent_configs() {
    let volume_parents = [config(1, 10, StagedFuzzySkinType::External)];
    let painted_parents = [config(2, 20, StagedFuzzySkinType::Hole)];

    let _ = staged_fuzzy_painted_region_configs(
        &volume_parents,
        &painted_parents,
        &[
            fuzzy_region(1, StagedFuzzyPaintedRegionParent::VolumeRegion(0), 101),
            fuzzy_region(2, StagedFuzzyPaintedRegionParent::PaintedRegion(0), 202),
        ],
    );

    assert_eq!(
        volume_parents,
        [config(1, 10, StagedFuzzySkinType::External)]
    );
    assert_eq!(painted_parents, [config(2, 20, StagedFuzzySkinType::Hole)]);
}
