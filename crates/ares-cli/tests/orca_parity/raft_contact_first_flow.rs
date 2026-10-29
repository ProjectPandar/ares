//! A first raft Contact uses interface flow, not the Base flange width.

#[test]
fn raft_contact_first_flow_no_raft_control() {
    crate::runner::native_stream_case::check(
        (
            "raft-contact-first-flow",
            &["Support/SupportCommon.cpp:1438-1441/1493-1504/1700-1738"],
        ),
        "control",
        include_bytes!("../../../../tests/parity/raft-contact-first-flow/control.3mf"),
        "4a868581914a8b5dde68862978fe2678401eb4c784a7ca0e6ccc32632343704e",
        (false, |_, _| {}),
    );
}

#[test]
fn raft_contact_first_flow_single_contact_positive() {
    crate::runner::native_stream_case::check(
        (
            "raft-contact-first-flow",
            &["Support/SupportCommon.cpp:1438-1441/1493-1504/1700-1738"],
        ),
        "positive",
        include_bytes!("../../../../tests/parity/raft-contact-first-flow/positive.3mf"),
        "e4f862f94000e30939c66a72fc94217826a4714f0fb7bf36f4280f431a91b706",
        (false, |_, _| {}),
    );
}
