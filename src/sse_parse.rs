//! Buffering Server-Sent Events (SSE) framing: `event:` + `data:` lines, blank line ends event.

use crate::stream_contract::StreamEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseParseError {
    Utf8(std::str::Utf8Error),
    EmptyEvent,
    MissingData,
    Decode(String),
}

impl std::fmt::Display for SseParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SseParseError::Utf8(e) => write!(f, "invalid utf-8 in SSE chunk: {e}"),
            SseParseError::EmptyEvent => write!(f, "SSE event without name"),
            SseParseError::MissingData => write!(f, "SSE event without data line"),
            SseParseError::Decode(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SseParseError {}

impl From<std::str::Utf8Error> for SseParseError {
    fn from(e: std::str::Utf8Error) -> Self {
        SseParseError::Utf8(e)
    }
}

/// Incrementally feed bytes; yields fully parsed [`StreamEvent`] values.
#[derive(Default, Debug)]
pub struct SseDecoder {
    buffer: String,
    pending_event: Option<String>,
    pending_data: Vec<String>,
}

impl SseDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push raw bytes from the HTTP body. Returns all events completed since last call.
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<StreamEvent>, SseParseError> {
        let s = std::str::from_utf8(chunk)?;
        self.buffer.push_str(s);
        self.drain()
    }

    /// Treat remaining buffer as end-of-stream: emit incomplete event if any (error).
    pub fn finish(self) -> Result<Vec<StreamEvent>, SseParseError> {
        if self.buffer.is_empty() && self.pending_event.is_none() && self.pending_data.is_empty() {
            return Ok(Vec::new());
        }
        Err(SseParseError::Decode(
            "incomplete SSE frame at end of stream".to_string(),
        ))
    }

    fn drain(&mut self) -> Result<Vec<StreamEvent>, SseParseError> {
        let mut out = Vec::new();
        while let Some(pos) = self.buffer.find('\n') {
            let mut line = self.buffer[..pos].to_string();
            self.buffer.drain(..=pos);
            if line.ends_with('\r') {
                line.pop();
            }
            self.process_line(&line, &mut out)?;
        }
        Ok(out)
    }

    fn process_line(
        &mut self,
        line: &str,
        out: &mut Vec<StreamEvent>,
    ) -> Result<(), SseParseError> {
        if line.is_empty() {
            return self.flush_event(out);
        }
        if line.starts_with(':') {
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("event:") {
            self.pending_event = Some(rest.trim().to_string());
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("data:") {
            self.pending_data.push(rest.to_string());
            return Ok(());
        }
        if line.starts_with("id:") || line.starts_with("retry:") {
            return Ok(());
        }
        Ok(())
    }

    fn flush_event(&mut self, out: &mut Vec<StreamEvent>) -> Result<(), SseParseError> {
        if self.pending_event.is_none() && self.pending_data.is_empty() {
            return Ok(());
        }
        let event_name = self
            .pending_event
            .take()
            .filter(|s| !s.is_empty())
            .ok_or(SseParseError::EmptyEvent)?;
        if self.pending_data.is_empty() {
            return Err(SseParseError::MissingData);
        }
        let payload = self.pending_data.join("\n");
        self.pending_data.clear();
        let ev = StreamEvent::from_wire(&event_name, &payload).map_err(SseParseError::Decode)?;
        out.push(ev);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_contract::{StreamDoneData, StreamTokenData};

    #[test]
    fn parses_single_token_event() {
        let mut d = SseDecoder::new();
        let evs = d
            .push(b"event: token\ndata: {\"text\":\"hi\",\"seq\":0}\n\n")
            .unwrap();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            StreamEvent::Token(t) => {
                assert_eq!(t.text, "hi");
                assert_eq!(t.seq, 0);
            }
            _ => panic!("expected token"),
        }
    }

    #[test]
    fn splits_chunk_mid_frame() {
        let mut d = SseDecoder::new();
        assert!(d.push(b"event: token\nda").unwrap().is_empty());
        let evs = d.push(b"ta: {\"text\":\"x\",\"seq\":1}\n\n").unwrap();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            StreamEvent::Token(t) => {
                assert_eq!(t.text, "x");
                assert_eq!(t.seq, 1);
            }
            _ => panic!("expected token"),
        }
    }

    #[test]
    fn ignores_comment_and_retry_lines() {
        let mut d = SseDecoder::new();
        let raw = b": hello\nretry: 3000\nevent: done\ndata: {\"seq\":2}\n\n";
        let evs = d.push(raw).unwrap();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            StreamEvent::Done(StreamDoneData { seq }) => assert_eq!(*seq, 2),
            _ => panic!("expected done"),
        }
    }

    #[test]
    fn multi_data_lines_joined() {
        let mut d = SseDecoder::new();
        d.push(b"event: token\ndata: {\"tex").unwrap();
        let evs = d
            .push(b"t\":\"ab\",\"seq\":0}\n\n")
            .expect("second chunk parses");
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            StreamEvent::Token(StreamTokenData { text, seq }) => {
                assert_eq!(text, "ab");
                assert_eq!(*seq, 0);
            }
            _ => panic!("expected token"),
        }
    }
}
