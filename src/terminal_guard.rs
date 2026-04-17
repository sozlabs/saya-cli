//! Restore terminal cursor and SGR on drop / panic so the shell stays usable.

/// Show cursor and reset graphic rendition (common after hiding cursor or styling).
pub fn terminal_reset() {
    // ?25h = show cursor, 0m = reset SGR
    let _ = std::io::Write::write_all(&mut std::io::stderr(), b"\x1b[?25h\x1b[0m");
    let _ = std::io::Write::flush(&mut std::io::stderr());
}

/// Runs [`terminal_reset`] when dropped (panic-safe cleanup hook for streaming UX).
pub struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        terminal_reset();
    }
}

/// Chain with the previous panic hook so default panic output is preserved.
pub fn install_panic_terminal_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        terminal_reset();
        previous(info);
    }));
}
