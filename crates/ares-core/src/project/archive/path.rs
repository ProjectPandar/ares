use crate::SliceError;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PackagePath(Box<str>);

impl PackagePath {
    pub(crate) fn entry(raw: &[u8]) -> Result<Self, SliceError> {
        let raw = raw.strip_prefix(b"/").unwrap_or(raw);
        let mut decoded = Vec::with_capacity(raw.len());
        let mut index = 0;
        while index < raw.len() {
            if raw[index] != b'%' {
                decoded.push(raw[index]);
                index += 1;
                continue;
            }

            let Some(encoded) = raw.get(index + 1..index + 3) else {
                return Err(invalid_path(raw, "has incomplete percent encoding"));
            };
            let Some(high) = hex_nibble(encoded[0]) else {
                return Err(invalid_path(raw, "has invalid percent encoding"));
            };
            let Some(low) = hex_nibble(encoded[1]) else {
                return Err(invalid_path(raw, "has invalid percent encoding"));
            };
            let byte = (high << 4) | low;
            if matches!(byte, b'/' | b'\\') {
                return Err(invalid_path(raw, "contains a percent-encoded separator"));
            }
            decoded.push(byte);
            index += 3;
        }

        let decoded = String::from_utf8(decoded)
            .map_err(|_| invalid_path(raw, "is not valid UTF-8 after percent decoding"))?;
        validate(raw, &decoded)?;
        Ok(Self(decoded.into()))
    }

    pub(crate) fn resolve(&self, target: &str) -> Result<Self, SliceError> {
        if target.starts_with('/') {
            return Self::entry(target.as_bytes());
        }

        let target = Self::entry(target.as_bytes())?;
        let Some((owner, _)) = self.0.rsplit_once('/') else {
            return Ok(target);
        };
        Ok(Self(format!("{owner}/{}", target.as_str()).into()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate(raw: &[u8], decoded: &str) -> Result<(), SliceError> {
    if decoded
        .split('/')
        .next()
        .is_some_and(|segment| segment.contains(':'))
    {
        return Err(invalid_path(raw, "uses drive or URI scheme syntax"));
    }
    if decoded.contains('\\') {
        return Err(invalid_path(raw, "contains a backslash"));
    }
    if decoded.contains('\0') {
        return Err(invalid_path(raw, "contains NUL"));
    }
    if decoded.contains('#') {
        return Err(invalid_path(raw, "contains a fragment"));
    }
    if decoded.contains('?') {
        return Err(invalid_path(raw, "contains a query"));
    }
    if decoded.split('/').any(|segment| segment.is_empty()) {
        return Err(invalid_path(raw, "contains an empty segment"));
    }
    if decoded
        .split('/')
        .any(|segment| matches!(segment, "." | ".."))
    {
        return Err(invalid_path(raw, "contains a dot segment"));
    }
    Ok(())
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn invalid_path(raw: &[u8], reason: &str) -> SliceError {
    SliceError::InvalidInput(format!(
        "invalid package path {:?}: {reason}",
        String::from_utf8_lossy(raw)
    ))
}
