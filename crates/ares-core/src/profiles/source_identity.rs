use std::fmt;

/// Opaque host-provided identity for one preset source file.
///
/// The core compares identities only. It neither serializes them nor interprets
/// their platform-specific representation.
#[derive(Clone, Eq, PartialEq)]
pub struct SourceIdentity(Box<[u8]>);

impl SourceIdentity {
    pub fn from_host_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        Self(bytes.into())
    }
}

impl fmt::Debug for SourceIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SourceIdentity(<opaque>)")
    }
}
