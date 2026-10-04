use super::*;

fn object() -> Object {
    Object {
        id: 7,
        copies: 2,
        flush_objects: false,
        flush_infill: true,
        flush_support: false,
        support_tools: [0, 0],
        support_interface_not_for_body: false,
        regions: Some(vec![Region {
            infill_first: false,
            wall_tool: 0,
            sparse_tool: 0,
            infills: vec![Collection {
                id: CollectionId::Infill(0),
                tool: 0,
                role: ExtrusionRole::InternalInfill,
                volume: 5.0,
            }],
            perimeters: Vec::new(),
        }]),
        support: None,
    }
}

#[test]
fn whole_collection_overrides_remain_independent_per_copy() {
    let objects = [object()];
    let layer = Layer {
        objects: &objects,
        tools: &[0, 1, 2],
        soluble: &[false; 3],
        support_filaments: &[false; 3],
    };
    let mut wiping = WipingExtrusions::new(&layer);
    assert_eq!(wiping.mark(&layer, (0, 1), 3.0), 0.0);
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 0), Some(1));
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 1), None);
    assert_eq!(wiping.mark(&layer, (1, 2), 3.0), 0.0);
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 0), Some(1));
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 1), Some(2));
}

#[test]
fn support_override_is_shared_between_copies_and_subtracts_double_volume() {
    let mut object = object();
    object.flush_infill = false;
    object.flush_support = true;
    object.support_tools = [0, 2];
    object.support = Some(vec![(ExtrusionRole::SupportMaterial, 0.999_999_98)]);
    let objects = [object];
    let layer = Layer {
        objects: &objects,
        tools: &[0, 1],
        soluble: &[false; 2],
        support_filaments: &[false; 2],
    };
    let mut wiping = WipingExtrusions::new(&layer);
    assert_eq!(
        wiping.mark(&layer, (0, 1), 1.0),
        (1.0_f64 - 0.999_999_98) as f32
    );
    assert_eq!(wiping.support_tool(7, false), Some(1));
    assert_eq!(wiping.support_tool(7, true), None);
}

#[test]
fn dedicated_objects_take_perimeters_before_other_objects_infill() {
    let mut dedicated = object();
    dedicated.id = 9;
    dedicated.copies = 1;
    dedicated.flush_objects = true;
    dedicated
        .regions
        .as_mut()
        .unwrap()
        .first_mut()
        .unwrap()
        .perimeters
        .push(Collection {
            id: CollectionId::Perimeter(0),
            tool: 0,
            role: ExtrusionRole::Perimeter,
            volume: 4.0,
        });
    let objects = [object(), dedicated];
    let layer = Layer {
        objects: &objects,
        tools: &[0, 1, 2],
        soluble: &[false; 3],
        support_filaments: &[false; 3],
    };
    let mut wiping = WipingExtrusions::new(&layer);
    assert_eq!(wiping.mark(&layer, (0, 1), 3.0), 0.0);
    assert_eq!(wiping.entity(9, CollectionId::Perimeter(0), 0), Some(1));
    assert_eq!(wiping.entity(9, CollectionId::Infill(0), 0), None);
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 0), None);
    assert_eq!(wiping.mark(&layer, (1, 2), 7.0), 0.0);
    assert_eq!(wiping.entity(9, CollectionId::Perimeter(0), 0), Some(1));
    assert_eq!(wiping.entity(9, CollectionId::Infill(0), 0), Some(2));
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 0), Some(2));
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 1), None);
}

#[test]
fn absent_nonsupport_tool_preserves_native_negative_one_normal_sentinel() {
    let mut object = object();
    object
        .regions
        .as_mut()
        .unwrap()
        .first_mut()
        .unwrap()
        .infill_first = true;
    let objects = [object];
    let layer = Layer {
        objects: &objects,
        tools: &[0],
        soluble: &[false],
        support_filaments: &[true],
    };
    let mut wiping = WipingExtrusions::new(&layer);
    wiping.ensure_order(&layer);
    // C++ converts unsigned(-1) to the signed "print normally" sentinel.
    assert!(wiping.has_overrides());
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 0), None);
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 1), None);
}

#[test]
fn ordering_preserves_the_source_boolean_tool_argument() {
    let objects = [object()];
    let layer = Layer {
        objects: &objects,
        tools: &[1, 0, 2],
        soluble: &[false; 3],
        support_filaments: &[false; 3],
    };
    let mut wiping = WipingExtrusions::new(&layer);
    wiping.ensure_order(&layer);
    // The source compares wall 0 with tool bool(2 || false) == 1, not tool 2.
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 0), None);
    assert_eq!(wiping.entity(7, CollectionId::Infill(0), 1), None);
}
