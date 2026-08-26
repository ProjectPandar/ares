use crate::SliceError;

use super::{compensation, perimeters, task22m_oracle, task22n_oracle};
#[cfg(test)]
use super::{
    prepare_post_closing, prepare_post_conical_overhang, prepare_post_largest_contours,
    prepare_post_regions, prepare_post_simplification, prepare_post_top_empty_layers,
    task22g_oracle, task22h_oracle, task22i_oracle, task22j_oracle,
};

#[cfg(test)]
pub fn task22g_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_closing(project)?;
    Ok(task22g_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22h_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_closing(project)?;
    Ok(task22g_oracle::encode_with_magic(
        &prepared.objects,
        b"ARES22G\0",
    ))
}

#[cfg(test)]
pub fn task22h_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_largest_contours(project)?;
    Ok(task22h_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22i_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_largest_contours(project)?;
    Ok(task22h_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22i_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_simplification(project)?;
    Ok(task22i_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22j_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_simplification(project)?;
    Ok(task22i_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22j_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_regions(project)?;
    Ok(task22j_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22k_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_regions(project)?;
    Ok(task22j_oracle::encode(&prepared.objects))
}

#[cfg(test)]
pub fn task22k_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_top_empty_layers(project)?;
    Ok(task22j_oracle::encode_with_magic(
        &prepared.objects,
        b"ARES22K\0",
    ))
}

#[cfg(test)]
pub fn task22l_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_top_empty_layers(project)?;
    let mut checkpoint = task22j_oracle::encode(&prepared.objects);
    checkpoint[..8].copy_from_slice(b"ARES22K\0");
    Ok(checkpoint)
}

#[cfg(test)]
pub fn task22l_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_conical_overhang(project)?;
    Ok(task22j_oracle::encode_with_magic(
        &prepared.objects,
        b"ARES22L\0",
    ))
}

#[cfg(test)]
pub fn task22m_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = prepare_post_conical_overhang(project)?;
    let mut checkpoint = task22j_oracle::encode(&prepared.objects);
    checkpoint[..8].copy_from_slice(b"ARES22L\0");
    Ok(checkpoint)
}

#[cfg(test)]
pub fn task22m_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = compensation::prepare_post_compensation(project)?;
    Ok(task22m_oracle::encode(&prepared.objects))
}

pub fn task22n_browser_input_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = compensation::prepare_post_compensation(project)?;
    Ok(task22m_oracle::encode(&prepared.objects))
}

pub fn task22n_browser_oracle(project: impl AsRef<[u8]>) -> Result<Vec<u8>, SliceError> {
    let prepared = compensation::prepare_post_compensation(project)?;
    let predecessor = task22m_oracle::encode(&prepared.objects);
    let prepared = perimeters::finish_post_perimeter_inputs(prepared)?;
    Ok(task22n_oracle::encode(&predecessor, &prepared.objects))
}
