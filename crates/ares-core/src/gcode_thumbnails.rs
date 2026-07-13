#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum GCodeThumbnailFormat {
    #[serde(rename = "PNG")]
    Png,
    #[serde(rename = "JPG")]
    Jpg,
    #[serde(rename = "QOI")]
    Qoi,
    #[serde(rename = "BTT_TFT")]
    BttTft,
    #[serde(rename = "COLPIC")]
    ColPic,
}

impl GCodeThumbnailFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpg => "JPG",
            Self::Qoi => "QOI",
            Self::BttTft => "BTT_TFT",
            Self::ColPic => "COLPIC",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GCodeThumbnailDefinition {
    pub format: GCodeThumbnailFormat,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThumbnailParseError {
    InvalidValue,
    OutOfRange,
    InvalidExtension,
}

pub fn parse_thumbnail_definitions(
    thumbnails: &str,
    default_extension: Option<&str>,
) -> Result<Vec<GCodeThumbnailDefinition>, ThumbnailParseError> {
    if thumbnails.is_empty() {
        return Ok(Vec::new());
    }

    let default_format = default_extension
        .and_then(GCodeThumbnailFormat::from_str)
        .unwrap_or(GCodeThumbnailFormat::Png);
    thumbnails
        .split(',')
        .map(|entry| parse_thumbnail_entry(entry, default_format))
        .collect()
}

impl GCodeThumbnailFormat {
    fn from_str(format: &str) -> Option<Self> {
        match format.to_ascii_uppercase().as_str() {
            "PNG" => Some(Self::Png),
            "JPG" => Some(Self::Jpg),
            "QOI" => Some(Self::Qoi),
            "BTT_TFT" => Some(Self::BttTft),
            "COLPIC" => Some(Self::ColPic),
            _ => None,
        }
    }
}

fn parse_thumbnail_entry(
    entry: &str,
    default_format: GCodeThumbnailFormat,
) -> Result<GCodeThumbnailDefinition, ThumbnailParseError> {
    let (width, rest) = entry
        .split_once('x')
        .ok_or(ThumbnailParseError::InvalidValue)?;
    let (height, extension) = rest
        .split_once('/')
        .map_or((rest, None), |(height, extension)| {
            (height, Some(extension))
        });
    let width = parse_dimension(width)?;
    let height = parse_dimension(height)?;
    let format = match extension {
        Some("") | None => default_format,
        Some(extension) => GCodeThumbnailFormat::from_str(extension)
            .ok_or(ThumbnailParseError::InvalidExtension)?,
    };
    Ok(GCodeThumbnailDefinition {
        format,
        width,
        height,
    })
}

fn parse_dimension(value: &str) -> Result<f64, ThumbnailParseError> {
    let value = value
        .trim()
        .parse::<f64>()
        .map_err(|_| ThumbnailParseError::InvalidValue)?;
    if value.is_finite() && value > 0.0 && value < 1000.0 {
        Ok(value)
    } else {
        Err(ThumbnailParseError::OutOfRange)
    }
}

pub const fn thumbnail_error_string(error: ThumbnailParseError) -> &'static str {
    match error {
        ThumbnailParseError::InvalidValue => {
            "Invalid input format. Expected vector of dimensions in the following format: \"XxY/EXT, XxY/EXT, ...\""
        }
        ThumbnailParseError::OutOfRange => "Input value is out of range",
        ThumbnailParseError::InvalidExtension => "Some extension in the input is invalid",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_upstream_thumbnail_formats_in_order() {
        let definitions =
            parse_thumbnail_definitions("1x2/png,3x4/jpg,5x6/qoi,7x8/btt_tft,9x10/colpic", None)
                .unwrap();

        assert_eq!(
            definitions,
            vec![
                GCodeThumbnailDefinition {
                    format: GCodeThumbnailFormat::Png,
                    width: 1.0,
                    height: 2.0,
                },
                GCodeThumbnailDefinition {
                    format: GCodeThumbnailFormat::Jpg,
                    width: 3.0,
                    height: 4.0,
                },
                GCodeThumbnailDefinition {
                    format: GCodeThumbnailFormat::Qoi,
                    width: 5.0,
                    height: 6.0,
                },
                GCodeThumbnailDefinition {
                    format: GCodeThumbnailFormat::BttTft,
                    width: 7.0,
                    height: 8.0,
                },
                GCodeThumbnailDefinition {
                    format: GCodeThumbnailFormat::ColPic,
                    width: 9.0,
                    height: 10.0,
                },
            ]
        );
        assert_eq!(GCodeThumbnailFormat::Png.as_str(), "PNG");
        assert_eq!(GCodeThumbnailFormat::Jpg.as_str(), "JPG");
        assert_eq!(GCodeThumbnailFormat::Qoi.as_str(), "QOI");
        assert_eq!(GCodeThumbnailFormat::BttTft.as_str(), "BTT_TFT");
        assert_eq!(GCodeThumbnailFormat::ColPic.as_str(), "COLPIC");
    }

    #[test]
    fn applies_default_extension_rules() {
        assert_eq!(
            parse_thumbnail_definitions("16x16", None).unwrap()[0].format,
            GCodeThumbnailFormat::Png
        );
        assert_eq!(
            parse_thumbnail_definitions("16x16", Some("qoi")).unwrap()[0].format,
            GCodeThumbnailFormat::Qoi
        );
        assert_eq!(
            parse_thumbnail_definitions("16x16", Some("unsupported")).unwrap()[0].format,
            GCodeThumbnailFormat::Png
        );
        assert_eq!(
            parse_thumbnail_definitions("16x16/", Some("jpg")).unwrap()[0].format,
            GCodeThumbnailFormat::Jpg
        );
    }

    #[test]
    fn returns_empty_definitions_for_empty_input() {
        assert_eq!(parse_thumbnail_definitions("", None).unwrap(), Vec::new());
    }

    #[test]
    fn distinguishes_thumbnail_parse_errors() {
        assert_eq!(
            parse_thumbnail_definitions("16", None),
            Err(ThumbnailParseError::InvalidValue)
        );
        assert_eq!(
            parse_thumbnail_definitions("0x16", None),
            Err(ThumbnailParseError::OutOfRange)
        );
        assert_eq!(
            parse_thumbnail_definitions("16x16/bmp", None),
            Err(ThumbnailParseError::InvalidExtension)
        );
        assert_eq!(
            parse_thumbnail_definitions("16x16/jpg ", None),
            Err(ThumbnailParseError::InvalidExtension)
        );
    }

    #[test]
    fn exposes_upstream_error_message_fragments() {
        assert!(
            thumbnail_error_string(ThumbnailParseError::InvalidValue)
                .contains("Invalid input format")
        );
        assert!(thumbnail_error_string(ThumbnailParseError::OutOfRange).contains("out of range"));
        assert!(
            thumbnail_error_string(ThumbnailParseError::InvalidExtension).contains("extension")
        );
    }
}
