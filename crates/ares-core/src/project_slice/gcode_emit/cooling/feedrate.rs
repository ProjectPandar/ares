pub(super) fn rewrite_layer(output: &mut Vec<u8>, layer_start: usize) {
    let layer = output.split_off(layer_start);
    let mut current_feedrate = 0_u64;
    for line in layer.split_inclusive(|byte| *byte == b'\n') {
        let Some(word) = feedrate_word(line) else {
            output.extend_from_slice(line);
            continue;
        };
        if word.value != current_feedrate {
            current_feedrate = word.value;
            output.extend_from_slice(line);
            continue;
        }
        if only_feedrate(line, word) {
            continue;
        }
        output.extend_from_slice(&line[..word.space]);
        output.extend_from_slice(&line[word.end..]);
    }
}

#[derive(Clone, Copy)]
struct FeedrateWord {
    space: usize,
    end: usize,
    value: u64,
}

fn feedrate_word(line: &[u8]) -> Option<FeedrateWord> {
    if line.len() < 3
        || line[0] != b'G'
        || !matches!(line[1], b'0' | b'1')
        || !line[2].is_ascii_whitespace()
    {
        return None;
    }
    let content_end = line
        .iter()
        .position(|byte| *byte == b';')
        .unwrap_or(line.len());
    let marker = line[2..content_end]
        .windows(2)
        .position(|window| window[0].is_ascii_whitespace() && window[1] == b'F')?
        + 2;
    let number_start = marker + 2;
    let end = line[number_start..content_end]
        .iter()
        .position(|byte| byte.is_ascii_whitespace())
        .map_or(content_end, |offset| number_start + offset);
    let integer_end = line[number_start..end]
        .iter()
        .position(|byte| !byte.is_ascii_digit())
        .map_or(end, |offset| number_start + offset);
    let value = line[number_start..integer_end]
        .iter()
        .fold(0_u64, |value, digit| value * 10 + u64::from(*digit - b'0'));
    Some(FeedrateWord {
        space: marker,
        end,
        value,
    })
}

fn only_feedrate(line: &[u8], word: FeedrateWord) -> bool {
    line[2..word.space].iter().all(u8::is_ascii_whitespace)
        && line[word.end..]
            .iter()
            .all(|byte| byte.is_ascii_whitespace() || *byte == b';')
}
