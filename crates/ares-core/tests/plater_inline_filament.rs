use ares_core::{FilamentTarget, PlaterSession};

#[test]
fn object_cell_clears_every_volume_override_while_the_context_action_keeps_the_modifier() {
    let fixture = include_bytes!("../../../tests/parity/modifier-feature-filament/control.3mf");
    let mut session = PlaterSession::load(fixture.to_vec()).unwrap();
    let object = &session.project().objects()[0];
    let parent = FilamentTarget {
        object: object.id(),
        volume: None,
    };
    let part = FilamentTarget {
        volume: Some(object.volumes()[0].id()),
        ..parent
    };
    let modifier = FilamentTarget {
        volume: Some(object.volumes()[1].id()),
        ..parent
    };
    let slots = |session: &PlaterSession| {
        let object = &session.project().objects()[0];
        let [part, modifier] = object.volumes() else {
            panic!("expected one normal part and one modifier");
        };
        (
            object.filament(),
            part.filament(),
            modifier.filament(),
            part.effective_filament(object),
            modifier.effective_filament(object),
        )
    };

    session.set_filament(&[part, modifier], (2, 2)).unwrap();
    let with_overrides = session.snapshot();
    assert_eq!(slots(&session), (Some(1), Some(2), Some(2), 2, 2));
    session.set_filament(&[parent], (2, 2)).unwrap();
    assert_eq!(slots(&session), (Some(2), None, Some(2), 2, 2));

    session.load_snapshot(&with_overrides);
    session.set_filament_cell(parent, (2, 2)).unwrap();
    assert_eq!(slots(&session), (Some(2), None, None, 2, 2));

    session.load_snapshot(&with_overrides);
    session.set_filament_cell(part, (1, 2)).unwrap();
    assert_eq!(slots(&session), (Some(1), Some(1), Some(2), 1, 2));
    session.set_filament_cell(modifier, (0, 2)).unwrap();
    let with_default = session.snapshot();
    assert_eq!(slots(&session), (Some(1), Some(1), Some(0), 1, 1));

    session.set_filament_cell(parent, (2, 2)).unwrap();
    let cleared = session.snapshot();
    session.load_snapshot(&with_default);
    assert_eq!(slots(&session), (Some(1), Some(1), Some(0), 1, 1));
    session.load_snapshot(&cleared);
    assert_eq!(slots(&session), (Some(2), None, None, 2, 2));
}
