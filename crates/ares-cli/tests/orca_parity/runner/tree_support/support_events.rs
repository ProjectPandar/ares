//! Support-role event extraction for the tree retraction witnesses.
//!
//! Every event is derived from the stream: which role a line belongs to comes
//! from its `;TYPE:` marker, nothing is stored.

const SUPPORT_ROLES: [&str; 3] = ["Support", "Support interface", "Support transition"];

/// Support-role events of one layer, all of them derived from the stream.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SupportEvents {
    /// Support loops extruded closed without a retraction inside.
    pub(super) closed_loops: usize,
    /// Support travels that resume extruding inside the same support role.
    pub(super) contained_travels: usize,
    /// Retractions out of a support interface into a wipe.
    pub(super) interface_wipes: usize,
}

fn is_support(role: &str) -> bool {
    SUPPORT_ROLES.contains(&role)
}

/// `(x, y, extrudes)` of a positioning `G1`, if the line has both axes.
fn move_point(line: &str) -> Option<(&str, &str, bool)> {
    if !line.starts_with("G1 ") {
        return None;
    }
    let mut x = None;
    let mut y = None;
    let mut extrudes = false;
    for word in line.split_whitespace() {
        if let Some(value) = word.strip_prefix('X') {
            x = Some(value);
        } else if let Some(value) = word.strip_prefix('Y') {
            y = Some(value);
        } else if word.starts_with('E') {
            extrudes = true;
        }
    }
    Some((x?, y?, extrudes))
}

fn nearest_extruding(lines: &[&str], index: usize) -> Option<usize> {
    lines[..index]
        .iter()
        .enumerate()
        .rev()
        .find_map(|(at, line)| move_point(line).is_some_and(|(_, _, e)| e).then_some(at))
}

fn next_move<'a>(lines: &'a [&'a str], mut index: usize) -> Option<(&'a str, &'a str, bool)> {
    while index < lines.len() {
        let line = lines[index];
        if line.starts_with("G1 Z") || line.starts_with("G1 E-") || line.starts_with(";TYPE:") {
            return None;
        }
        if line.starts_with("G1 F") || line.starts_with(";") {
            index += 1;
            continue;
        }
        return move_point(line);
    }
    None
}

fn layer_events<'a>(lines: &[&'a str], role: &mut &'a str) -> SupportEvents {
    let mut roles: Vec<&'a str> = Vec::with_capacity(lines.len());
    for line in lines {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            *role = value;
        }
        roles.push(*role);
    }
    let mut events = SupportEvents::default();
    let mut loop_start: Option<(&str, &str)> = None;
    let mut loop_length = 0;
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("G1 F") || line.starts_with(";") {
            continue;
        }
        if line.starts_with("G1 E-") {
            let wipe = lines.get(index + 1) == Some(&";WIPE_START")
                && lines[..index]
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(at, line)| {
                        move_point(line).is_some_and(|(_, _, e)| e).then_some(at)
                    })
                    .is_some_and(|at| roles[at] == "Support interface");
            if wipe {
                events.interface_wipes += 1;
            }
            loop_start = None;
            continue;
        }
        let Some((x, y, extrudes)) = move_point(line) else {
            loop_start = None;
            continue;
        };
        if !extrudes {
            if is_support(roles[index]) && matches!(next_move(lines, index + 1), Some((_, _, true)))
            {
                events.contained_travels += 1;
            }
            loop_start = is_support(roles[index]).then_some((x, y));
            loop_length = 0;
            continue;
        }
        let Some((sx, sy)) = loop_start else { continue };
        if (x, y) == (sx, sy) && loop_length >= 3 {
            events.closed_loops += 1;
        }
        loop_length += 1;
    }
    events
}

/// Per-layer support events; the role state carries across layers.
pub(super) fn support_events(stream: &str) -> Vec<SupportEvents> {
    let mut role = "";
    stream
        .split(";LAYER_CHANGE")
        .skip(1)
        .map(|layer| layer_events(&layer.lines().collect::<Vec<_>>(), &mut role))
        .collect()
}

#[derive(Clone, Copy)]
pub(super) enum Mutation {
    Loop,
    Travel,
    Wipe,
}

/// Mutates the support events of `stream`: with `in_scope` only inside support
/// roles, otherwise only outside them.
pub(super) fn mutate(stream: &str, mutation: Mutation, in_scope: bool) -> String {
    let lines: Vec<&str> = stream.lines().collect();
    let mut role = "";
    let mut roles: Vec<&str> = Vec::with_capacity(lines.len());
    for line in &lines {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        roles.push(role);
    }
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        let scoped = is_support(roles[index]) == in_scope;
        out.push((*line).to_owned());
        match mutation {
            Mutation::Wipe => {
                let boundary = line == &";WIPE_START"
                    && scoped
                    && nearest_extruding(&lines, index)
                        .is_some_and(|at| roles[at] == "Support interface");
                if boundary {
                    out.last_mut().unwrap().push_str("_MUTATED");
                }
            }
            Mutation::Travel => {
                let Some((_, _, extrudes)) = move_point(line) else {
                    continue;
                };
                if extrudes {
                    continue;
                }
                let mut at = index + 1;
                while at < lines.len()
                    && (lines[at].starts_with("G1 F") || lines[at].starts_with(";"))
                {
                    at += 1;
                }
                if scoped
                    && lines
                        .get(at)
                        .is_some_and(|line| move_point(line).is_some_and(|(_, _, e)| e))
                {
                    out.push("G1 E-2.8 F3600".to_owned());
                }
            }
            Mutation::Loop => {
                if line.starts_with("G1 F") || line.starts_with(";") {
                    continue;
                }
                if line.starts_with("G1 E-") {
                    continue;
                }
                let Some((_, _, extrudes)) = move_point(line) else {
                    continue;
                };
                if !extrudes {
                    scoped.then(|| out.push("G1 X0 Y0 F9000".to_owned()));
                    continue;
                }
            }
        }
    }
    out.join("\n")
}
