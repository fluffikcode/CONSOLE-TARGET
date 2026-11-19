//! System telemetry panel shown once at startup.

use std::{
    io::{self, Write},
    path::Path,
    thread,
    time::Duration,
};

use super::theme::*;

/// Print the startup checklist and the session summary.
pub fn print_dashboard(workdir: &Path, model: &str, base_url: &str, has_key: bool) {
    let checks = [
        "MOUNTING VFS SANDBOX...",
        "LOCKING DIRECTORY BOUNDARIES...",
        "BINDING NEURAL INTERFACE...",
    ];
    for check in checks {
        print!("\r   {GRAY}▪ {check}{RESET}\x1b[K");
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(65));
    }
    print!("\r\x1b[2K");

    let auth_badge = if has_key {
        format!("{INV_WHITE} AUTH: KEY {RESET}")
    } else {
        format!("{GRAY}[AUTH: LOCAL/NONE]{RESET}")
    };

    println!("   {GRAY}╭─[ {SILVER}SYSTEM TELEMETRY{GRAY} ]──────────────────────────────╮{RESET}");
    println!(
        "   {GRAY}│{RESET} {GRAY}MODEL   :{RESET} {WHITE}{BOLD}{:<24}{RESET} {:>14} {GRAY}│{RESET}",
        model.trim(),
        auth_badge
    );
    println!(
        "   {GRAY}│{RESET} {GRAY}ENDPOINT:{RESET} {SILVER}{:<39}{RESET} {GRAY}│{RESET}",
        truncate_middle(base_url, 39)
    );
    println!(
        "   {GRAY}│{RESET} {GRAY}SANDBOX :{RESET} {WHITE}{:<39}{RESET} {GRAY}│{RESET}",
        truncate_middle(&workdir.display().to_string(), 39)
    );
    println!("   {GRAY}├───────────────────────────────────────────────────┤{RESET}");
    println!(
        "   {GRAY}│{RESET} {GRAY}COMMANDS:{RESET} {WHITE}/clear{RESET} {GRAY}screen ·{RESET} {WHITE}/reset{RESET} {GRAY}memory ·{RESET} {WHITE}/exit{RESET}   {GRAY}│{RESET}"
    );
    println!("   {GRAY}╰───────────────────────────────────────────────────╯{RESET}\n");
}

/// Shorten a path from the left so the most specific segment stays visible.
fn truncate_middle(value: &str, max: usize) -> String {
    let length = value.chars().count();
    if length <= max {
        return value.to_string();
    }

    let tail: String = value
        .chars()
        .rev()
        .take(max.saturating_sub(3))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("...{tail}")
}
