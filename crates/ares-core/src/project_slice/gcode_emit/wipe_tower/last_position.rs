//! `GCodeProcessor.cpp:4310–4399`: terminal XYZ and any-axis parse success.

pub(super) fn scan(source: &str, position: &mut [f32; 3]) -> bool {
    let mut changed = false;
    for line in source
        .split_inclusive('\n')
        .filter_map(|line| line.strip_suffix('\n'))
    {
        let line = line
            .trim_matches(' ')
            .trim_end_matches(';')
            .trim_end_matches(' ');
        if line.len() <= 5
            || !["G0 ", "G1 ", "G2 ", "G3 "]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        {
            continue;
        }
        for (axis, key) in [" X", " Y", " Z"].iter().enumerate() {
            let Some((_, tail)) = line.split_once(key) else {
                continue;
            };
            let word = tail.split([' ', '\t', '\r', '\n', ';']).next().unwrap();
            if word.len() <= 8
                && let Ok(value) = word.parse::<f32>()
            {
                position[axis] = value;
                changed = true;
            }
        }
    }
    changed
}
