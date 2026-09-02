use crate::{
    GCodeThumbnailDefinition, GCodeThumbnailFormat, ThumbnailDefinitions, ThumbnailParseError,
};

pub(crate) fn normalize_thumbnails(
    thumbnails: &ThumbnailDefinitions,
    default_format: Option<GCodeThumbnailFormat>,
) -> Result<ThumbnailDefinitions, ThumbnailParseError> {
    let definitions = parse_fixed_definitions(thumbnails.as_str(), default_format)?;
    if definitions.is_empty() {
        return Ok(thumbnails.clone());
    }

    // `PrintConfigDef::handle_legacy_composite` (`PrintConfig.cpp:8290-8322`)
    // rewrites a loaded non-empty value into the canonical
    // "{w}x{h}/{EXT}" list joined by ", " before the config dump echoes it.
    Ok(ThumbnailDefinitions(crate::OrcaString(
        definitions
            .iter()
            .map(|definition| {
                format!(
                    "{}x{}/{}",
                    format_dimension(definition.width),
                    format_dimension(definition.height),
                    definition.format.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join(", "),
    )))
}

fn parse_fixed_definitions(
    thumbnails: &str,
    default_format: Option<GCodeThumbnailFormat>,
) -> Result<Vec<GCodeThumbnailDefinition>, ThumbnailParseError> {
    if thumbnails.is_empty() {
        return Ok(Vec::new());
    }
    thumbnails
        .split_terminator(',')
        .map(|entry| parse_fixed_entry(entry, default_format.unwrap_or(GCodeThumbnailFormat::Png)))
        .collect()
}

fn parse_fixed_entry(
    entry: &str,
    default_format: GCodeThumbnailFormat,
) -> Result<GCodeThumbnailDefinition, ThumbnailParseError> {
    let (width, rest) = entry
        .split_once('x')
        .filter(|(width, _)| !width.is_empty())
        .ok_or(ThumbnailParseError::InvalidValue)?;
    let (height, extension) = rest
        .split_once('/')
        .map_or((rest, None), |(height, extension)| {
            (
                height,
                Some(extension.split('/').next().unwrap_or_default()),
            )
        });
    if height.is_empty() {
        return Err(ThumbnailParseError::InvalidValue);
    }

    let width = parse_stream_number(width);
    let height = parse_stream_number(height);
    if !dimension_in_range(width) || !dimension_in_range(height) {
        return Err(ThumbnailParseError::OutOfRange);
    }

    let format = match extension {
        None | Some("") => default_format,
        Some(extension) => parse_format(extension)?,
    };
    Ok(GCodeThumbnailDefinition {
        format,
        width,
        height,
    })
}

fn parse_stream_number(value: &str) -> f64 {
    numeric_prefix(value)
        .and_then(|prefix| prefix.parse().ok())
        .unwrap_or(0.0)
}

fn numeric_prefix(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    let mut start = 0;
    while bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
        start += 1;
    }
    let mut end = start;
    if bytes
        .get(end)
        .is_some_and(|byte| matches!(byte, b'+' | b'-'))
    {
        end += 1;
    }

    let integer_start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut has_digit = end != integer_start;
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        let fraction_start = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        has_digit |= end != fraction_start;
    }
    if !has_digit {
        return None;
    }

    if bytes
        .get(end)
        .is_some_and(|byte| matches!(byte, b'e' | b'E'))
    {
        end += 1;
        if bytes
            .get(end)
            .is_some_and(|byte| matches!(byte, b'+' | b'-'))
        {
            end += 1;
        }
        let digits = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == digits {
            return None;
        }
    }
    Some(&value[start..end])
}

fn dimension_in_range(value: f64) -> bool {
    value.is_finite() && value > 0.0 && value < 1000.0
}

fn parse_format(extension: &str) -> Result<GCodeThumbnailFormat, ThumbnailParseError> {
    match extension.to_ascii_uppercase().as_str() {
        "PNG" => Ok(GCodeThumbnailFormat::Png),
        "JPG" => Ok(GCodeThumbnailFormat::Jpg),
        "QOI" => Ok(GCodeThumbnailFormat::Qoi),
        "BTT_TFT" => Ok(GCodeThumbnailFormat::BttTft),
        "COLPIC" => Ok(GCodeThumbnailFormat::ColPic),
        _ => Err(ThumbnailParseError::InvalidExtension),
    }
}

fn format_dimension(value: f64) -> String {
    let scientific = format!("{value:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    if !(-4..6).contains(&exponent) {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!("{mantissa}e{exponent:+03}")
    } else {
        let precision = usize::try_from(5 - exponent).unwrap();
        let rounded: f64 = scientific.parse().unwrap();
        format!("{rounded:.precision$}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    }
}
