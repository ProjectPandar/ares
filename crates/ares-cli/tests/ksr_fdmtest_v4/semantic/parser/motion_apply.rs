//! Motion application: apply parsed `G1` moves to the semantic model
//! (depositions, wipes, travels).

use super::super::model::Position;
use super::motion::Motion;
use super::{Deposition, Layer, LifecycleEvent, MotionRecord, SemanticGcode, State, Travel};
use std::collections::BTreeMap;

pub(super) fn apply_motion(
    output: &mut SemanticGcode,
    state: &mut State,
    motion: Motion,
) -> Result<(), String> {
    if let Some(feed) = motion.values.get(&'F') {
        state.feed.clone_from(feed);
    }
    let next_x = motion.values.get(&'X').unwrap_or(&state.x).clone();
    let next_y = motion.values.get(&'Y').unwrap_or(&state.y).clone();
    let next_z = motion.values.get(&'Z').unwrap_or(&state.z).clone();
    let next = [next_x.as_str(), next_y.as_str(), next_z.as_str()];
    let moved_xy = next_x != state.x || next_y != state.y;
    let moved_z = next_z != state.z;
    if let (Some(layer), Some(extrusion)) = (output.layers.last_mut(), motion.values.get(&'E')) {
        let value = extrusion
            .parse::<f64>()
            .map_err(|_| format!("invalid extrusion {extrusion:?}"))?;
        if value > 0.0 && moved_xy {
            layer.deposition.push(Deposition {
                feature: state.feature.clone(),
                width: state.width.clone(),
                motion: motion_record(state, &motion, next),
                extrusion: extrusion.clone(),
                feed: required_feed(state)?,
                acceleration: state.print_acceleration.clone(),
                fans: fan_state(&state.fans),
            });
        } else if value < 0.0 && moved_xy {
            push_lifecycle(
                layer,
                LifecycleEvent::Wipe {
                    motion: Box::new(motion_record(state, &motion, next)),
                    extrusion: extrusion.clone(),
                    feed: state.feed.clone(),
                },
            );
        } else {
            let event = LifecycleEvent::Extruder {
                extrusion: extrusion.clone(),
                feed: state.feed.clone(),
            };
            if value > 0.0 {
                layer.lifecycles.push(vec![event]);
            } else {
                push_lifecycle(layer, event);
            }
        }
    } else if let Some(layer) = output.layers.last_mut()
        && (moved_xy || moved_z)
    {
        layer.travels.push(Travel {
            motion: motion_record(state, &motion, next),
            feed: required_feed(state)?,
            acceleration: state.travel_acceleration.clone(),
        });
    }
    state.x = next_x;
    state.y = next_y;
    state.z = next_z;
    Ok(())
}

pub(super) fn push_lifecycle(layer: &mut Layer, event: LifecycleEvent) {
    if layer.lifecycles.is_empty() {
        layer.lifecycles.push(Vec::new());
    }
    layer.lifecycles.last_mut().unwrap().push(event);
}

fn motion_record(state: &State, motion: &Motion, next: [&str; 3]) -> MotionRecord {
    MotionRecord {
        plane: state.plane.clone(),
        command: motion.command.clone(),
        start: Position {
            x: state.x.clone(),
            y: state.y.clone(),
            z: state.z.clone(),
        },
        end: Position {
            x: next[0].to_owned(),
            y: next[1].to_owned(),
            z: next[2].to_owned(),
        },
        arc_center: [
            motion.values.get(&'I').cloned(),
            motion.values.get(&'J').cloned(),
        ],
        turns: motion.values.get(&'P').cloned(),
    }
}

fn required_feed(state: &State) -> Result<f64, String> {
    state
        .feed
        .parse::<f64>()
        .map_err(|_| "deposition move has no feed rate".to_owned())
}

fn fan_state(fans: &BTreeMap<String, String>) -> String {
    fans.iter()
        .map(|(fan, speed)| format!("{fan}:{speed}"))
        .collect::<Vec<_>>()
        .join(",")
}
