use quick_xml::XmlVersion;

// XML 1.0 and XML 1.1 section 2.2 define different Char productions.
pub(super) fn is_legal_character(character: char, version: XmlVersion) -> bool {
    match version {
        XmlVersion::Explicit1_1 => matches!(
            character,
            '\u{1}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
        ),
        XmlVersion::Implicit1_0 | XmlVersion::Explicit1_0 => matches!(
            character,
            '\u{9}' | '\u{a}' | '\u{d}' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
        ),
    }
}

// XML 1.1 RestrictedChar values are legal only when written as character references.
pub(super) fn is_xml11_restricted(character: char) -> bool {
    matches!(
        character,
        '\u{1}'..='\u{8}' | '\u{b}'..='\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}'..='\u{84}' | '\u{86}'..='\u{9f}'
    )
}
