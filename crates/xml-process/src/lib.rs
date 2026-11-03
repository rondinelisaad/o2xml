//! Shared hardening for supervised helper processes.

use tokio::process::Command;

/// Prevents helper processes from opening a transient console window on Windows.
///
/// SciXML launches validation and transformation helpers in the background. A GUI
/// application must set `CREATE_NO_WINDOW`; otherwise Windows briefly creates a
/// console for every validation cycle.
pub fn hide_console(command: &mut Command) {
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);

    #[cfg(not(windows))]
    let _ = command;
}
