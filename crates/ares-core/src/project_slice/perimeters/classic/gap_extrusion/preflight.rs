use crate::{SliceError, geometry::CoordinateScale};

use super::super::medial_gap::PreparedPostClassicMedialGap;

pub(super) struct ValidatedObject {
    pub(super) records: Vec<Option<ValidatedRecord>>,
}

#[derive(Clone, Copy)]
pub(super) struct ValidatedRecord {
    pub(super) threshold: f64,
    pub(super) flow: crate::project_slice::perimeters::types::Flow,
}

pub(super) fn validate(
    prepared: &PreparedPostClassicMedialGap,
    scale: CoordinateScale,
) -> Result<Vec<ValidatedObject>, SliceError> {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(source, traversal)| {
            let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
            let inputs = prelude.object.as_parts().1;
            assert_eq!(source.records.len(), inputs.len());
            let records = source
                .records
                .iter()
                .zip(inputs)
                .map(|(source, input)| match (source, input) {
                    (None, None) => Ok(None),
                    (Some(_), Some(input)) => {
                        let value = prelude.object.region_options(input).filter_out_gap_fill.0;
                        Ok(Some(ValidatedRecord {
                            threshold: validate_threshold(value, scale)?,
                            flow: input.solid_infill_flow,
                        }))
                    }
                    _ => panic!("O14 input record alignment is invariant"),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ValidatedObject { records })
        })
        .collect()
}

pub(super) fn validate_threshold(value: f64, scale: CoordinateScale) -> Result<f64, SliceError> {
    if !value.is_finite() || value < 0.0 {
        return Err(SliceError::InvalidInput(
            "invalid Orca option filter_out_gap_fill".to_owned(),
        ));
    }
    Ok(value / scale.factor())
}
