use quick_xml::{Reader, events::Event};

pub(super) fn object_record_permutation(name: &str, left: &[u8], right: &[u8]) -> bool {
    name == "Metadata/model_settings.config"
        && model_settings_parts(left)
            .zip(model_settings_parts(right))
            .is_some_and(|(left, right)| left == right)
}

#[derive(PartialEq, Eq)]
struct ModelSettingsParts<'a> {
    records: Vec<&'a [u8]>,
    outside: Vec<&'a [u8]>,
}

fn model_settings_parts(bytes: &[u8]) -> Option<ModelSettingsParts<'_>> {
    // bbs_3mf.cpp:6218,8306: Native iterates complete object records by pointer.
    let mut reader = Reader::from_reader(bytes);
    let mut records = Vec::new();
    let mut outside = Vec::new();
    let mut saw_config = false;
    let mut depth = 0usize;
    let mut start = None;
    let mut last_end = 0;
    loop {
        let offset = reader.buffer_position() as usize;
        match (depth, reader.read_event().ok()?) {
            (0, Event::Start(tag)) if !saw_config && tag.name().as_ref() == b"config" => {
                saw_config = true;
                depth = 1;
            }
            (0, Event::Start(_)) => return None,
            (1, Event::Start(tag)) if tag.name().as_ref() == b"object" => {
                start = Some(offset);
                depth = 2;
            }
            (_, Event::Start(_)) => depth += 1,
            (2, Event::End(_)) if start.is_some() => {
                let start = start.take()?;
                let gap = &bytes[last_end..start];
                if !records.is_empty() && !gap.iter().all(u8::is_ascii_whitespace) {
                    return None;
                }
                let end = reader.buffer_position() as usize;
                outside.push(gap);
                records.push(&bytes[start..end]);
                last_end = end;
                depth = 1;
            }
            (_, Event::End(_)) => depth = depth.checked_sub(1)?,
            (_, Event::Eof) => break,
            _ => {}
        }
    }
    if depth != 0 || records.len() < 2 {
        return None;
    }
    outside.push(&bytes[last_end..]);
    records.sort_unstable();
    Some(ModelSettingsParts { records, outside })
}
