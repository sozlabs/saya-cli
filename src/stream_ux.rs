//! Stderr UX for SSE `emotion` / `status` events (never mixed into assistant `token` text).

use crate::stream_contract::{StreamEmotionData, StreamStatusData};
use std::io::Write;
use std::time::{Duration, Instant};

const DIM: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Tracks semantic emotion visibility: higher `priority` or expired TTL replaces the active emotion.
pub struct StreamUx {
    emotion_active: Option<(i64, Instant)>,
}

impl StreamUx {
    pub fn new() -> Self {
        Self {
            emotion_active: None,
        }
    }

    pub fn on_emotion(&mut self, e: &StreamEmotionData, now: Instant) {
        let take = match self.emotion_active {
            None => true,
            Some((prio, deadline)) => now >= deadline || e.priority > prio,
        };
        if !take {
            return;
        }
        self.emotion_active = Some((e.priority, now + Duration::from_millis(e.ttl_ms.max(1))));
        let _ = writeln!(
            std::io::stderr(),
            "{DIM}[saya:emotion]{RESET} {:?} priority={} ttl_ms={} seq={}",
            e.state,
            e.priority,
            e.ttl_ms,
            e.seq
        );
    }

    pub fn on_status(&self, s: &StreamStatusData) {
        let detail = s.detail.as_deref().unwrap_or("");
        let _ = writeln!(
            std::io::stderr(),
            "{DIM}[saya:status]{RESET} {:?} seq={} {detail}",
            s.status,
            s.seq
        );
    }

    #[cfg(test)]
    pub fn active_emotion_priority(&self) -> Option<i64> {
        self.emotion_active.map(|(p, _)| p)
    }
}

impl Default for StreamUx {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_contract::{EmotionSource, EmotionState};

    fn emotion(state: EmotionState, prio: i64, ttl: u64, seq: u64) -> StreamEmotionData {
        StreamEmotionData {
            state,
            source: EmotionSource::Semantic,
            priority: prio,
            ttl_ms: ttl,
            event_id: "e".into(),
            conversation_id: "c".into(),
            ts: String::new(),
            seq,
        }
    }

    #[test]
    fn skips_lower_priority_until_deadline() {
        let mut ux = StreamUx::new();
        let t0 = Instant::now();
        ux.on_emotion(&emotion(EmotionState::Focus, 5, 1000, 0), t0);
        ux.on_emotion(
            &emotion(EmotionState::Dormant, 4, 1000, 1),
            t0 + Duration::from_millis(10),
        );
        assert_eq!(ux.active_emotion_priority(), Some(5));
    }

    #[test]
    fn replaces_after_ttl_even_if_lower_priority() {
        let mut ux = StreamUx::new();
        let t0 = Instant::now();
        ux.on_emotion(&emotion(EmotionState::Focus, 5, 50, 0), t0);
        ux.on_emotion(
            &emotion(EmotionState::Happy, 1, 100, 1),
            t0 + Duration::from_millis(60),
        );
        assert_eq!(ux.active_emotion_priority(), Some(1));
    }

    #[test]
    fn higher_priority_preempts() {
        let mut ux = StreamUx::new();
        let t0 = Instant::now();
        ux.on_emotion(&emotion(EmotionState::Focus, 5, 1000, 0), t0);
        ux.on_emotion(
            &emotion(EmotionState::Confused, 10, 1000, 1),
            t0 + Duration::from_millis(10),
        );
        assert_eq!(ux.active_emotion_priority(), Some(10));
    }
}
