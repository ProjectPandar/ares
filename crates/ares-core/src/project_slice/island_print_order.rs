use crate::{
    geometry::Point,
    project_slice::{
        extrusion_islands::{ExtrusionIsland, IslandInfillEntity, PreparedPostExtrusionIslands},
        fill_entities::{FillExtrusionCollection, FillExtrusionEntity},
        perimeters::classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
            shortest_path::ChainEntity,
        },
    },
};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) enum IslandPrintEntity {
    Perimeter(ExtrusionEntityCollection),
    Fill(FillExtrusionEntity),
    FillCollection(FillExtrusionCollection),
    Thin(GapFillEntity),
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct OrderedExtrusionIsland {
    pub(in crate::project_slice) entities: Vec<IslandPrintEntity>,
    pub(in crate::project_slice) infill_regions: Vec<usize>,
    pub(in crate::project_slice) infill_collections: Vec<usize>,
    pub(in crate::project_slice) perimeter_regions: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::project_slice) struct OrderedExtrusionLayer {
    pub(in crate::project_slice) islands: Vec<OrderedExtrusionIsland>,
    pub(in crate::project_slice) tool_regions: Vec<super::extrusion_islands::RegionToolUsage>,
}

pub(in crate::project_slice) struct PreparedPostIslandPrintOrder {
    pub(in crate::project_slice) predecessor: PreparedPostExtrusionIslands,
    pub(in crate::project_slice) objects: Vec<Vec<OrderedExtrusionLayer>>,
    pub(in crate::project_slice) support_layers:
        Vec<Vec<super::support_material::InstalledSupportLayer>>,
    /// Top surfaces per object per layer — the classified kinds from the
    /// surface-type stage (`get_boundary`'s top subtraction,
    /// `AvoidCrossingPerimeters.cpp:1122-1132`).
    pub(in crate::project_slice) top_surfaces: Vec<Vec<Vec<crate::geometry::ExPolygon>>>,
    /// Per-object per-layer seam penalty data for the Nearest seam mode
    /// (`SeamPlacer.cpp:930-940` `pick_nearest_seam_point_index` —
    /// overhang, embedded distance, visibility + angle penalty per
    /// candidate, ready for the emit-time gaussian distance penalty).
    pub(in crate::project_slice) nearest_seam_plans: Vec<Vec<NearestSeamLayer>>,
}

impl PreparedPostIslandPrintOrder {
    /// The resolved filament config the emit pass printed with.
    pub(in crate::project_slice) fn filament(&self) -> &crate::FilamentGCodeSourceOptions {
        &self
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .resolved
            .views
            .full
            .filament
            .gcode
    }
}

/// Per-layer candidate data for the Nearest seam selection at emit time
/// (`SeamPlacer.cpp:1500-1560` `place_seam` with spNearest — geometric
/// loop-to-perimeter match, then gaussian distance penalty minimization
/// per `pick_nearest_seam_point_index`).
pub(in crate::project_slice) struct NearestSeamLayer {
    /// Unscaled candidate positions `(x, y, z)` — upstream `SeamCandidate::
    /// position` is a `Vec3f` whose z is the layer `slice_z`
    /// (`SeamPlacer.cpp:1025,1509`); the KD tree partitions on all three
    /// axes (`KDTreeIndirect<3>`), so z must be stored even though it is
    /// constant within a layer.
    pub(in crate::project_slice) positions: Vec<(f32, f32, f32)>,
    /// Penalty score per candidate: `visibility +
    /// ANGLE_IMPORTANCE_NEAREST * angle_penalty` (`SeamPlacer.cpp:788-791`).
    pub(in crate::project_slice) scores: Vec<f32>,
    /// Overhang distance per candidate (`SeamPlacer.cpp:765-767`).
    pub(in crate::project_slice) overhangs: Vec<f32>,
    pub(in crate::project_slice) unsupported_distances: Option<Vec<f32>>,
    /// Embedded (hidden-inside) distance per candidate
    /// (`SeamCandidate::embedded_distance`, `SeamPlacer.cpp:769-775`).
    pub(in crate::project_slice) embedded: Vec<f32>,
    /// Perimeter point ranges `(start_index, end_index)` per perimeter
    /// (`SeamCandidate::perimeter`).
    pub(in crate::project_slice) perimeter_ranges: Vec<(usize, usize)>,
    /// Per-candidate perimeter index (candidate → perimeter).
    pub(in crate::project_slice) perimeter_of_candidate: Vec<usize>,
    /// Per-candidate local ccw angle in radians (`SeamCandidate::local_ccw_angle`).
    pub(in crate::project_slice) ccw_angles: Vec<f32>,
    /// The layer's candidate KD tree — built once and reused across
    /// perimeter loops like upstream `SeamPlacer` (`SeamPlacer.cpp:
    /// 1021-1036` builds one tree per layer).
    pub(in crate::project_slice) kdtree:
        std::cell::OnceCell<crate::project_slice::seam_placement::kdtree::KdTree>,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostExtrusionIslands,
    top_surfaces: Vec<Vec<Vec<crate::geometry::ExPolygon>>>,
) -> PreparedPostIslandPrintOrder {
    let nearest_seam_plans: Vec<Vec<NearestSeamLayer>> = Vec::new();
    let infill_first = {
        let traversal = &predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        (0..traversal.objects.len())
            .map(|index| {
                traversal.print_object_config(index).0.layer_candidates[0].model_parts[0]
                    .region
                    .is_infill_first
                    .0
            })
            .collect::<Vec<_>>()
    };
    let objects = predecessor
        .objects
        .iter_mut()
        .enumerate()
        .map(|(object_index, layers)| {
            layers
                .iter_mut()
                .enumerate()
                .map(|(layer_index, layer)| OrderedExtrusionLayer {
                    tool_regions: std::mem::take(&mut layer.tool_regions),
                    islands: std::mem::take(&mut layer.islands)
                        .into_iter()
                        .map(|island| {
                            order_island(island, layer_index == 0, infill_first[object_index])
                        })
                        .collect(),
                })
                .collect()
        })
        .collect();
    PreparedPostIslandPrintOrder {
        predecessor,
        objects,
        top_surfaces,
        nearest_seam_plans,
        support_layers: Vec::new(),
    }
}

pub(in crate::project_slice) fn order_island(
    island: ExtrusionIsland,
    first_layer: bool,
    infill_first: bool,
) -> OrderedExtrusionIsland {
    let ExtrusionIsland {
        infills,
        perimeters,
    } = island;
    let (perimeter_regions, perimeters): (Vec<_>, Vec<_>) = perimeters.into_iter().unzip();
    let perimeters = perimeters.into_iter().map(IslandPrintEntity::Perimeter);
    let mut infill_regions = Vec::with_capacity(infills.len());
    let mut infill_collections = Vec::with_capacity(infills.len());
    let infills = infills.into_iter().map(|(region, collection, entity)| {
        infill_regions.push(region);
        infill_collections.push(collection);
        match entity {
            IslandInfillEntity::Fill(entity) => IslandPrintEntity::Fill(entity),
            IslandInfillEntity::FillCollection(collection) => {
                IslandPrintEntity::FillCollection(collection)
            }
            IslandInfillEntity::Thin(entity) => IslandPrintEntity::Thin(entity),
        }
    });
    let entities = if !first_layer && infill_first {
        infills.chain(perimeters).collect()
    } else {
        perimeters.chain(infills).collect()
    };
    OrderedExtrusionIsland {
        entities,
        infill_regions,
        infill_collections,
        perimeter_regions,
    }
}

impl ChainEntity for IslandPrintEntity {
    fn first_point(&self) -> Point {
        match self {
            Self::Fill(entity) => entity.first_point(),
            Self::FillCollection(collection) => collection.first_point(),
            Self::Thin(entity) => entity.first_point(),
            Self::Perimeter(_) => unreachable!("only infill entities are chained"),
        }
    }

    fn last_point(&self) -> Point {
        match self {
            Self::Fill(entity) => entity.last_point(),
            Self::FillCollection(collection) => collection.last_point(),
            Self::Thin(entity) => entity.last_point(),
            Self::Perimeter(_) => unreachable!("only infill entities are chained"),
        }
    }

    fn can_reverse(&self) -> bool {
        match self {
            Self::Fill(entity) => entity.can_reverse(),
            Self::FillCollection(collection) => !collection.no_sort,
            Self::Thin(entity) => ChainEntity::can_reverse(entity),
            Self::Perimeter(_) => unreachable!("only infill entities are chained"),
        }
    }

    fn reverse(&mut self) {
        match self {
            Self::Fill(entity) => entity.reverse(),
            Self::FillCollection(collection) => collection.reverse(),
            Self::Thin(entity) => entity.reverse(),
            Self::Perimeter(_) => unreachable!("only infill entities are chained"),
        }
    }
}

pub(in crate::project_slice) fn internal_surfaces(
    prepared: &PreparedPostExtrusionIslands,
    object_index: usize,
    layer_index: usize,
) -> Vec<&crate::project_slice::region_slices::RegionSurface> {
    let external_surfaces = &prepared.predecessor.predecessor.predecessor.predecessor;
    let horizontal = &external_surfaces.predecessor;
    let layer_count = horizontal.predecessor.objects[object_index]
        .physical_layers()
        .len();
    horizontal.objects[object_index]
        .records
        .chunks_exact(layer_count)
        .filter_map(|records| records[layer_index].as_ref())
        .flat_map(|record| &record.slices)
        .collect()
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostIslandPrintOrder) {
    super::extrusion_islands::dispose(prepared.predecessor);
}
