use quick_xml::{
    NsReader, XmlVersion,
    events::{BytesStart, Event},
    name::ResolveResult,
};
use serde::de::DeserializeOwned;

use crate::SliceError;

use super::xml_characters::{is_legal_character, is_xml11_restricted};

mod attribute;
mod model;
mod role;

pub(crate) use role::{JsonRole, XmlRole};

const MAX_DOCUMENT_SIZE: usize = 64 * 1024 * 1024;
const MAX_DEPTH: usize = 256;
const MAX_ATTRIBUTES: usize = 1_024;
const MAX_DECODED_TEXT: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy)]
struct XmlLimits {
    document_size: usize,
    decoded_text: usize,
}

impl XmlLimits {
    const PROJECT: Self = Self {
        document_size: MAX_DOCUMENT_SIZE,
        decoded_text: MAX_DECODED_TEXT,
    };
}

pub(crate) fn deserialize_xml<T: DeserializeOwned>(
    input: &[u8],
    role: XmlRole,
) -> Result<T, SliceError> {
    validate_xml(input, role, XmlLimits::PROJECT)?;
    let mut deserializer = quick_xml::de::Deserializer::borrowing(configured_reader(input));
    T::deserialize(&mut deserializer).map_err(|error| invalid_xml(role, error))
}

pub(crate) fn deserialize_json<T: DeserializeOwned>(
    input: &[u8],
    role: JsonRole,
) -> Result<T, SliceError> {
    if input.len() > MAX_DOCUMENT_SIZE {
        return Err(SliceError::InvalidInput(format!(
            "invalid project {} JSON: document exceeds {MAX_DOCUMENT_SIZE} bytes",
            role.name()
        )));
    }
    serde_json::from_slice(input).map_err(|error| {
        SliceError::InvalidInput(format!("invalid project {} JSON: {error}", role.name()))
    })
}

#[cfg(test)]
pub(crate) fn validate_xml_for_test(
    input: &[u8],
    role: XmlRole,
    document_size: usize,
    decoded_text: usize,
) -> Result<(), SliceError> {
    validate_xml(
        input,
        role,
        XmlLimits {
            document_size,
            decoded_text,
        },
    )
}

fn configured_reader(input: &[u8]) -> NsReader<&[u8]> {
    let mut reader = NsReader::from_reader(input);
    reader.config_mut().enable_all_checks(true);
    reader
        .resolver_mut()
        .set_max_declarations_per_element(MAX_ATTRIBUTES);
    reader
}

fn validate_xml(input: &[u8], role: XmlRole, limits: XmlLimits) -> Result<(), SliceError> {
    if input.len() > limits.document_size {
        return Err(invalid_xml(
            role,
            format_args!("document exceeds {} bytes", limits.document_size),
        ));
    }

    let mut reader = configured_reader(input);
    let mut depth = 0usize;
    let mut decoded_text = DecodedText::new(limits.decoded_text);
    let mut root_seen = false;
    let mut root_closed = false;
    let mut declaration_seen = false;
    let mut version = XmlVersion::Implicit1_0;

    loop {
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| invalid_xml(role, error))?;
        match event {
            Event::Start(element) => {
                if depth == 0 {
                    begin_root(role, &namespace, &element, &mut root_seen, root_closed)?;
                } else {
                    validate_namespace(role, &namespace)?;
                }
                validate_attributes(role, &reader, &element, version, &mut decoded_text)?;
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| invalid_xml(role, "element depth overflows"))?;
                if depth > MAX_DEPTH {
                    return Err(invalid_xml(
                        role,
                        format_args!("element depth exceeds {MAX_DEPTH}"),
                    ));
                }
            }
            Event::Empty(element) => {
                if depth == 0 {
                    begin_root(role, &namespace, &element, &mut root_seen, root_closed)?;
                    root_closed = true;
                } else {
                    validate_namespace(role, &namespace)?;
                }
                validate_attributes(role, &reader, &element, version, &mut decoded_text)?;
            }
            Event::End(_) => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| invalid_xml(role, "unmatched closing element"))?;
                if depth == 0 {
                    root_closed = true;
                }
            }
            Event::Text(text) => {
                let literal = text.decode().map_err(|error| invalid_xml(role, error))?;
                validate_literal_characters(role, &literal, version)?;
                let text = text
                    .xml_content(version)
                    .map_err(|error| invalid_xml(role, error))?;
                decoded_text.add(role, text.len())?;
            }
            Event::CData(text) => {
                let literal = text.decode().map_err(|error| invalid_xml(role, error))?;
                validate_literal_characters(role, &literal, version)?;
                let text = text
                    .xml_content(version)
                    .map_err(|error| invalid_xml(role, error))?;
                decoded_text.add(role, text.len())?;
            }
            Event::GeneralRef(reference) => {
                let len = match reference
                    .resolve_char_ref()
                    .map_err(|error| invalid_xml(role, error))?
                {
                    Some(character) => {
                        validate_referenced_character(role, character, version)?;
                        character.len_utf8()
                    }
                    None if is_predefined_reference(&reference) => 1,
                    None => return Err(invalid_xml(role, "general entity reference is forbidden")),
                };
                decoded_text.add(role, len)?;
            }
            Event::DocType(_) => {
                return Err(invalid_xml(
                    role,
                    "document type declarations are forbidden",
                ));
            }
            Event::Decl(declaration) => {
                if declaration_seen || root_seen {
                    return Err(invalid_xml(role, "misplaced or repeated XML declaration"));
                }
                version = declaration
                    .xml_version()
                    .map_err(|error| invalid_xml(role, error))?;
                declaration_seen = true;
            }
            Event::Eof => break,
            Event::Comment(_) | Event::PI(_) => {}
        }
    }

    if !root_seen || !root_closed || depth != 0 {
        return Err(invalid_xml(role, "missing or unclosed root element"));
    }
    Ok(())
}

fn begin_root(
    role: XmlRole,
    namespace: &ResolveResult<'_>,
    element: &BytesStart<'_>,
    root_seen: &mut bool,
    root_closed: bool,
) -> Result<(), SliceError> {
    if *root_seen || root_closed {
        return Err(invalid_xml(role, "multiple root elements"));
    }
    if element.local_name().as_ref() != role.root() {
        return Err(invalid_xml(role, "unexpected root element"));
    }
    validate_namespace(role, namespace)?;
    *root_seen = true;
    Ok(())
}

fn validate_namespace(role: XmlRole, namespace: &ResolveResult<'_>) -> Result<(), SliceError> {
    let matches = match (role.namespace(), namespace) {
        (Some(expected), ResolveResult::Bound(actual)) => actual.as_ref() == expected,
        (None, ResolveResult::Unbound) => true,
        _ => false,
    };
    if !matches {
        return Err(invalid_xml(role, "unexpected or unresolved XML namespace"));
    }
    Ok(())
}

fn validate_attributes(
    role: XmlRole,
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    version: XmlVersion,
    decoded_text: &mut DecodedText,
) -> Result<(), SliceError> {
    let mut required_extensions = None;
    let mut namespace_bindings = Vec::new();
    for (index, attribute) in element.attributes().enumerate() {
        if index >= MAX_ATTRIBUTES {
            return Err(invalid_xml(
                role,
                format_args!("element has more than {MAX_ATTRIBUTES} attributes"),
            ));
        }
        let attribute = attribute.map_err(|error| invalid_xml(role, error))?;
        let (namespace, local_name) = reader.resolver().resolve_attribute(attribute.key);
        if matches!(namespace, ResolveResult::Unknown(_)) {
            return Err(invalid_xml(role, "attribute uses an unresolved namespace"));
        }
        attribute::validate_namespace(
            role,
            attribute.key.as_ref(),
            local_name.as_ref(),
            &namespace,
        )?;
        let literal = reader
            .decoder()
            .decode(attribute.value.as_ref())
            .map_err(|error| invalid_xml(role, error))?;
        validate_literal_characters(role, &literal, version)?;
        let value = attribute
            .decoded_and_normalized_value(version, reader.decoder())
            .map_err(|error| invalid_xml(role, error))?;
        validate_legal_characters(role, &value, version)?;
        decoded_text.add(role, value.len())?;
        if role == XmlRole::Model && attribute.key.as_ref() == b"requiredextensions" {
            required_extensions = Some(value.into_owned());
        } else if role == XmlRole::Model
            && let Some(prefix) = attribute.key.as_ref().strip_prefix(b"xmlns:")
        {
            namespace_bindings.push((prefix.to_vec(), value.into_owned()));
        }
    }
    if role == XmlRole::Model && element.local_name().as_ref() == b"model" {
        model::validate_required_extensions(required_extensions.as_deref(), &namespace_bindings)?;
    }
    Ok(())
}

struct DecodedText {
    total: usize,
    limit: usize,
}

impl DecodedText {
    fn new(limit: usize) -> Self {
        Self { total: 0, limit }
    }

    fn add(&mut self, role: XmlRole, len: usize) -> Result<(), SliceError> {
        self.total = self
            .total
            .checked_add(len)
            .ok_or_else(|| invalid_xml(role, "decoded text length overflows"))?;
        if self.total > self.limit {
            return Err(invalid_xml(
                role,
                format_args!("decoded text exceeds {} bytes", self.limit),
            ));
        }
        Ok(())
    }
}

fn is_predefined_reference(reference: &[u8]) -> bool {
    reference == b"lt"
        || reference == b"gt"
        || reference == b"amp"
        || reference == b"apos"
        || reference == b"quot"
}

fn validate_literal_characters(
    role: XmlRole,
    value: &str,
    version: XmlVersion,
) -> Result<(), SliceError> {
    if value.chars().all(|character| {
        is_legal_character(character, version)
            && (version != XmlVersion::Explicit1_1 || !is_xml11_restricted(character))
    }) {
        return Ok(());
    }
    Err(invalid_xml(role, "literal contains a forbidden character"))
}

fn validate_legal_characters(
    role: XmlRole,
    value: &str,
    version: XmlVersion,
) -> Result<(), SliceError> {
    if value
        .chars()
        .all(|character| is_legal_character(character, version))
    {
        return Ok(());
    }
    Err(invalid_xml(
        role,
        "character reference is not a legal XML character",
    ))
}

fn validate_referenced_character(
    role: XmlRole,
    character: char,
    version: XmlVersion,
) -> Result<(), SliceError> {
    if is_legal_character(character, version) {
        return Ok(());
    }
    Err(invalid_xml(
        role,
        "character reference is not a legal XML character",
    ))
}

fn invalid_xml(role: XmlRole, reason: impl std::fmt::Display) -> SliceError {
    SliceError::InvalidInput(format!("invalid project {} XML: {reason}", role.name()))
}
