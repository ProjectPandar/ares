use super::{
    CoolingLine, TYPE_ADJUSTABLE, TYPE_EXTERNAL_PERIMETER, TYPE_EXTRUDE_END, TYPE_HAS_F, TYPE_WIPE,
};

const SET_SPEED: &[u8] = b";_EXTRUDE_SET_SPEED";
const EXTERNAL_PERIMETER: &[u8] = b";_EXTERNAL_PERIMETER";
const WIPE: &[u8] = b";_WIPE";

pub(super) fn append(output: &mut Vec<u8>, gcode: &[u8], lines: &mut [CoolingLine]) {
    lines.sort_unstable_by_key(|line| line.start);
    let mut position = 0;
    let mut current_feedrate = 0;

    for line in lines {
        if line.start > position {
            output.extend_from_slice(&gcode[position..line.start]);
        }
        let source = &gcode[line.start..line.end];
        if line.kind & TYPE_EXTRUDE_END != 0 {
            position = line.end;
            continue;
        }
        if line.kind & (TYPE_ADJUSTABLE | TYPE_EXTERNAL_PERIMETER | TYPE_WIPE | TYPE_HAS_F) == 0 {
            output.extend_from_slice(source);
            position = line.end;
            continue;
        }

        let comment_start = source
            .iter()
            .position(|byte| *byte == b';')
            .unwrap_or(source.len());
        let command = &source[..comment_start];
        let (word_start, value_start, value_end) =
            feedrate_word(command).expect("cooling line has a feedrate word");
        let source_feedrate = parse_integer(&command[value_start..value_end]);
        let new_feedrate = if line.slowed {
            (60.0 * f64::from(line.feedrate) + 0.5).floor() as i32
        } else {
            source_feedrate
        };

        if new_feedrate == current_feedrate {
            if line.kind & (TYPE_ADJUSTABLE | TYPE_EXTERNAL_PERIMETER | TYPE_WIPE) != 0 {
                position = line.end;
                continue;
            }
            if line.length == 0.0 {
                output.extend_from_slice(&command[value_end..]);
                position = line.end;
                continue;
            }
            output.extend_from_slice(&command[..word_start]);
            output.extend_from_slice(&command[value_end..]);
        } else if line.slowed {
            output.extend_from_slice(&command[..value_start]);
            output.extend_from_slice(new_feedrate.to_string().as_bytes());
            output.extend_from_slice(&command[value_end..]);
            current_feedrate = new_feedrate;
        } else {
            output.extend_from_slice(command);
            current_feedrate = new_feedrate;
        }

        if comment_start < source.len() {
            if line.kind & (TYPE_ADJUSTABLE | TYPE_EXTERNAL_PERIMETER | TYPE_WIPE) != 0 {
                append_clean_comment(output, &source[comment_start..], line.kind);
            } else {
                output.extend_from_slice(&source[comment_start..]);
            }
        }
        position = line.end;
    }

    if position < gcode.len() {
        output.extend_from_slice(&gcode[position..]);
    }
}

fn feedrate_word(command: &[u8]) -> Option<(usize, usize, usize)> {
    let mut index = 2.min(command.len());
    while index + 1 < command.len() {
        if command[index] == b' ' && command[index + 1] == b'F' {
            let value_start = index + 2;
            let mut value_end = value_start;
            while value_end < command.len() && !command[value_end].is_ascii_whitespace() {
                value_end += 1;
            }
            let mut word_start = index;
            while word_start > 0 && command[word_start - 1].is_ascii_whitespace() {
                word_start -= 1;
            }
            return Some((word_start, value_start, value_end));
        }
        index += 1;
    }
    None
}

fn parse_integer(value: &[u8]) -> i32 {
    let length = value
        .iter()
        .position(|byte| !byte.is_ascii_digit() && *byte != b'-' && *byte != b'+')
        .unwrap_or(value.len());
    std::str::from_utf8(&value[..length])
        .expect("feedrate is ASCII")
        .parse()
        .expect("feedrate starts with an integer")
}

fn append_clean_comment(output: &mut Vec<u8>, comment: &[u8], kind: u32) {
    let mut position = 0;
    while position < comment.len() {
        let marker = [
            Some(SET_SPEED),
            (kind & TYPE_EXTERNAL_PERIMETER != 0).then_some(EXTERNAL_PERIMETER),
            (kind & TYPE_WIPE != 0).then_some(WIPE),
        ]
        .into_iter()
        .flatten()
        .filter_map(|marker| find(&comment[position..], marker).map(|offset| (offset, marker)))
        .min_by_key(|(offset, _)| *offset);
        let Some((offset, marker)) = marker else {
            output.extend_from_slice(&comment[position..]);
            return;
        };
        output.extend_from_slice(&comment[position..position + offset]);
        position += offset + marker.len();
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
