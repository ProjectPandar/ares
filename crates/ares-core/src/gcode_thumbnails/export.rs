use super::{
    GCodeThumbnailDefinition, GCodeThumbnailFormat, ThumbnailData, render, thumbnail_error_string,
};
use crate::{CancelToken, Project, ProjectSettings, SliceError};

fn definitions(value: &str) -> Result<Vec<GCodeThumbnailDefinition>, SliceError> {
    crate::options::parse_loaded_thumbnail_definitions(value, None).map_err(|error| {
        SliceError::InvalidInput(format!(
            "Invalid thumbnails value:\n - {}",
            thumbnail_error_string(error)
        ))
    })
}

/// `Thumbnails.hpp:60-64`: render one callback result per configured definition.
pub(crate) fn append_rendered(
    output: &mut Vec<u8>,
    project: &Project,
    settings: &ProjectSettings,
    cancel: &CancelToken,
) -> Result<(), SliceError> {
    let definitions = definitions(settings.printer.remaining.thumbnails.as_str())?;
    if definitions.is_empty() {
        return Ok(());
    }
    let Some(scene) = render::Scene::prepare(project, settings, cancel)? else {
        return Ok(());
    };
    let mut first_colpic = true;
    for (position, definition) in definitions.iter().enumerate() {
        let width = definition.width.round() as u32;
        let height = definition.height.round() as u32;
        if let Some(image) = scene.image([width, height], cancel)? {
            append_image(
                output,
                &image,
                definition.format,
                position + 1 == definitions.len(),
                &mut first_colpic,
            )?;
        }
    }
    Ok(())
}

/// `Thumbnails.hpp::export_thumbnails_to_file`, supplied by caller-rendered frames.
pub(crate) fn append(
    output: &mut Vec<u8>,
    value: &str,
    images: &[ThumbnailData],
) -> Result<(), SliceError> {
    if images.is_empty() {
        return Ok(());
    }
    let definitions = definitions(value)?;
    let mut first_colpic = true;
    for (position, definition) in definitions.iter().enumerate() {
        for image in images.iter().filter(|image| {
            f64::from(image.width()) == definition.width.trunc()
                && f64::from(image.height()) == definition.height.trunc()
        }) {
            append_image(
                output,
                image,
                definition.format,
                position + 1 == definitions.len(),
                &mut first_colpic,
            )?;
        }
    }
    Ok(())
}

fn append_image(
    output: &mut Vec<u8>,
    image: &ThumbnailData,
    format: GCodeThumbnailFormat,
    last: bool,
    first_colpic: &mut bool,
) -> Result<(), SliceError> {
    match format {
        GCodeThumbnailFormat::BttTft => super::btt::append(image, output, last),
        GCodeThumbnailFormat::Qoi => {
            append_base64(output, image, "thumbnail_QOI", &super::qoi::encode(image))
        }
        GCodeThumbnailFormat::Png => {
            append_base64(output, image, "thumbnail", &super::png::encode(image))
        }
        GCodeThumbnailFormat::ColPic => {
            super::colpic::append(image, output, *first_colpic)?;
            *first_colpic = false;
        }
        GCodeThumbnailFormat::Jpg => {
            append_base64(output, image, "thumbnail_JPG", &super::jpg::encode(image))
        }
    }
    Ok(())
}

fn append_base64(output: &mut Vec<u8>, image: &ThumbnailData, tag: &str, compressed: &[u8]) {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = Vec::with_capacity(compressed.len().div_ceil(3) * 4);
    for chunk in compressed.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        encoded.push(ALPHABET[usize::from(first >> 2)]);
        encoded.push(ALPHABET[usize::from((first & 3) << 4 | second >> 4)]);
        encoded.push(if chunk.len() > 1 {
            ALPHABET[usize::from((second & 15) << 2 | third >> 6)]
        } else {
            b'='
        });
        encoded.push(if chunk.len() > 2 {
            ALPHABET[usize::from(third & 63)]
        } else {
            b'='
        });
    }
    output.extend_from_slice(b"; THUMBNAIL_BLOCK_START\n");
    output.extend_from_slice(
        format!(
            "\n;\n; {tag} begin {}x{} {}\n",
            image.width(),
            image.height(),
            encoded.len()
        )
        .as_bytes(),
    );
    for row in encoded.chunks(78) {
        output.extend_from_slice(b"; ");
        output.extend_from_slice(row);
        output.push(b'\n');
    }
    output.extend_from_slice(format!("; {tag} end\n; THUMBNAIL_BLOCK_END\n\n").as_bytes());
}
