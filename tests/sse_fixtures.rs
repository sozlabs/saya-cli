//! Golden-style SSE framing checks against checked-in fixtures.

use saya_cli::sse_parse::SseDecoder;
use saya_cli::stream_contract::StreamEvent;

#[test]
fn mixed_events_fixture_chunked_arbitrarily() {
    let raw = include_str!("fixtures/sse/mixed_events.txt");
    let mut decoder = SseDecoder::new();
    let mut all = Vec::new();
    for chunk in raw.as_bytes().chunks(11) {
        all.extend(decoder.push(chunk).expect("parse chunk"));
    }
    assert_eq!(all.len(), 6);
    assert!(matches!(all[0], StreamEvent::Emotion(_)));
    match &all[1] {
        StreamEvent::Token(t) => assert_eq!(t.text, "hello "),
        _ => panic!("expected token"),
    }
    match &all[2] {
        StreamEvent::Token(t) => assert_eq!(t.text, "world"),
        _ => panic!("expected token"),
    }
    assert!(matches!(all[3], StreamEvent::Status(_)));
    assert!(matches!(all[4], StreamEvent::Tool(_)));
    assert!(matches!(all[5], StreamEvent::Done(_)));
}
