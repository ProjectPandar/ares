// Semver.hpp::parse/to_string; deps_src/semver/semver.c::semver_parse_version.
pub(crate) fn parse(value: &str) -> Option<String> {
    if value.len() > 255
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
    {
        return None;
    }
    let (number, metadata) = value
        .split_once('+')
        .map_or((value, None), |(number, suffix)| (number, Some(suffix)));
    let (number, prerelease) = number
        .split_once('-')
        .map_or((number, None), |(number, suffix)| (number, Some(suffix)));
    let mut parts = [0_i32; 4];
    let mut count = 0;
    for part in number.split('.') {
        if part.len() > 50 {
            return None;
        }
        *parts.get_mut(count)? = if part.is_empty() {
            0
        } else {
            part.parse().ok()?
        };
        count += 1;
    }
    if count < 2 {
        return None;
    }
    let patch = if count == 4 {
        parts[2].checked_mul(100)?.checked_add(parts[3])?
    } else {
        parts[2]
    };
    let mut result = format!("{}.{}.{}.{}", parts[0], parts[1], patch / 100, patch % 100);
    if let Some(suffix) = prerelease {
        result.push('-');
        result.push_str(suffix);
    }
    if let Some(suffix) = metadata {
        result.push('+');
        result.push_str(suffix);
    }
    Some(result)
}

pub(crate) fn is_newer(value: &str, current: &str) -> bool {
    let Some(value) = major_minor(value) else {
        return false;
    };
    let Some(current) = major_minor(current) else {
        return false;
    };
    value > current
}

fn major_minor(value: &str) -> Option<(i32, i32)> {
    let normalized = parse(value)?;
    let mut parts = normalized.split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}
