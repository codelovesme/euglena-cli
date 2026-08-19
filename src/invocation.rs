//! How to spell "this program" in the CLI's own printed hints.
//!
//! When cdlvsm dispatches to an installed package it sets
//! `CDLVSM_INVOKED_AS="cdlvsm <pkg>"` (see codelovesme/cdlvsm-cli). Under
//! cdlvsm's default install there is no bare `euglena` command — only a
//! `cdlvsm-euglena` shim — so a hint that prints a literal `euglena run` is
//! wrong for most users. Reading this env var back lets every hint say the
//! command that will actually work for however the user invoked us.

/// The command prefix to show in hints, e.g. `"cdlvsm euglena"` when
/// dispatched through cdlvsm, or `"euglena"` for a direct/non-cdlvsm install.
pub fn command_prefix() -> String {
    std::env::var("CDLVSM_INVOKED_AS")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "euglena".to_string())
}
