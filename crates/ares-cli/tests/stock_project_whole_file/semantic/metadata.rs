#[derive(Debug, Default)]
pub(crate) struct Timing {
    pub(crate) model: u64,
    pub(crate) total: u64,
    pub(crate) first_layer: u64,
}

pub(super) fn parse_timing(line: &str, timing: &mut Timing) -> Result<bool, String> {
    if let Some(value) = line.strip_prefix(";TIME:") {
        let seconds = value
            .parse::<f64>()
            .map_err(|_| "invalid legacy TIME line".to_owned())?;
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("invalid legacy TIME line".to_owned());
        }
        timing.model = seconds.round() as u64;
        timing.total = timing.model;
        return Ok(true);
    }
    if let Some(value) = line.strip_prefix("; model printing time: ") {
        let (model, total) = value
            .split_once("; total estimated time: ")
            .ok_or_else(|| "invalid model printing time line".to_owned())?;
        timing.model = duration_seconds(model)?;
        timing.total = duration_seconds(total)?;
        return Ok(true);
    }
    if let Some(value) = line.strip_prefix("; estimated printing time (normal mode) = ") {
        timing.model = duration_seconds(value)?;
        timing.total = timing.model;
        return Ok(true);
    }
    if let Some((_, value)) = line
        .strip_prefix("; estimated first layer printing time ")
        .and_then(|value| value.split_once("= "))
    {
        timing.first_layer = duration_seconds(value)?;
        return Ok(true);
    }
    Ok(false)
}

fn duration_seconds(value: &str) -> Result<u64, String> {
    let mut seconds = 0.0_f64;
    for part in value.split_whitespace() {
        let (digits, multiplier) = match part.as_bytes().last() {
            Some(b'h') => (&part[..part.len() - 1], 3_600.0),
            Some(b'm') => (&part[..part.len() - 1], 60.0),
            Some(b's') => (&part[..part.len() - 1], 1.0),
            _ => return Err(format!("invalid duration {value:?}")),
        };
        seconds += digits
            .parse::<f64>()
            .map_err(|_| format!("invalid duration {value:?}"))?
            * multiplier;
    }
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(format!("invalid duration {value:?}"));
    }
    Ok(seconds.round() as u64)
}

pub(super) fn parse_filament_lengths(line: &str) -> Result<Option<Vec<f64>>, String> {
    if let Some(value) = line
        .strip_prefix(";Filament used:")
        .and_then(|value| value.strip_suffix('m'))
    {
        let meters = value
            .parse::<f64>()
            .map_err(|_| format!("invalid legacy filament length {value:?}"))?;
        if !meters.is_finite() || meters < 0.0 {
            return Err(format!("invalid legacy filament length {value:?}"));
        }
        return Ok(Some(vec![meters * 1_000.0]));
    }
    let Some(value) = line.strip_prefix("; filament used [mm] = ") else {
        return Ok(None);
    };
    value
        .split(',')
        .map(|value| {
            value
                .trim()
                .parse::<f64>()
                .map_err(|_| format!("invalid filament length {value:?}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

pub(super) fn ignored_postamble_stat(line: &str) -> bool {
    [
        "; filament used [cm3] = ",
        "; filament used [g] = ",
        "; filament cost = ",
        "; total filament used [g] = ",
        "; total filament cost = ",
        "; total layers count = ",
        "; estimated printing time ",
        "; estimated first layer printing time ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}
