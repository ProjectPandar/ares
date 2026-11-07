pub(super) fn object_start(text: &str) -> usize {
    let object = regex::Regex::new(r#"<object\b[^>]*>"#).unwrap();
    let starts = object.find_iter(text).collect::<Vec<_>>();
    assert_eq!(starts.len(), 1, "expected exactly one object element");
    let attribute = regex::Regex::new(r#"([A-Za-z_][A-Za-z0-9_.:-]*)\s*=\s*"([^"]*)""#).unwrap();
    let ids = attribute
        .captures_iter(starts[0].as_str())
        .filter(|capture| capture.get(1).unwrap().as_str() == "id")
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 1, "expected exactly one object id attribute");
    assert!(
        !ids[0].get(2).unwrap().as_str().is_empty(),
        "object id must not be empty"
    );
    starts[0].end()
}

pub(super) fn part_end(text: &str) -> usize {
    let part = regex::Regex::new(r#"<part id="[^"]+" subtype="normal_part">"#).unwrap();
    let starts = part.find_iter(text).collect::<Vec<_>>();
    assert_eq!(starts.len(), 1);
    starts[0].end()
}
