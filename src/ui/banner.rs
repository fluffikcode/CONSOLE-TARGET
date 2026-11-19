//! Animated startup banner and screen clearing.

use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use super::theme::*;

const RAW_LOGO: &[&str] = &[
    "████████╗ █████╗ ██████╗  ██████╗ ███████╗████████╗",
    "╚══██╔══╝██╔══██╗██╔══██╗██╔════╝ ██╔════╝╚══██╔══╝",
    "   ██║   ███████║██████╔╝██║  ███╗█████╗     ██║   ",
    "   ██║   ██╔══██║██╔══██╗██║   ██║██╔══╝     ██║   ",
    "   ██║   ██║  ██║██║  ██║╚██████╔╝███████╗   ██║   ",
    "   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ",
];

/// Print the animated TARGET wordmark and version plate.
pub fn print_banner() {
    println!();
    let scanline = "▓▓▒▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░▒▒▓▓";
    for line in RAW_LOGO {
        print!("\r   {GRAY}{DIM}{scanline}{RESET}");
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(20));

        let colored = line.replace('█', &format!("{WHITE}{BOLD}█{RESET}{GRAY}"));
        println!("\r   {GRAY}{colored}{RESET}\x1b[K");
        thread::sleep(Duration::from_millis(15));
    }

    println!();
    println!("   {GRAY}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓{RESET}");
    println!(
        "   {GRAY}┃{RESET} {INV_WHITE} T A R G E T {RESET}  {SILVER}LOCAL AUTONOMOUS FILE AGENT{RESET}  {GRAY}v0.1{RESET} {GRAY}┃{RESET}"
    );
    println!("   {GRAY}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛{RESET}");
}

/// Clear the terminal and move the cursor to the home position.
pub fn clear_screen() {
    print!("\x1b[2J\x1b[1;1H");
    let _ = io::stdout().flush();
}
