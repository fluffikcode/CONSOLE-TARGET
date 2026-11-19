//! Interactive terminal implementation of the write-approval gate.

use std::io::{self, Write};

use anyhow::Result;

use crate::approval::{Approver, WriteRequest};

use super::theme::*;

/// Shows a preview of the pending write and asks for `y`/`N` consent.
#[derive(Debug, Default, Clone, Copy)]
pub struct TerminalApprover;

impl Approver for TerminalApprover {
    fn approve_write(&self, request: &WriteRequest<'_>) -> Result<bool> {
        println!();
        println!(
            "   {GRAY}┏━━━{RESET}{INV_WHITE} ⚠ WRITE INTERCEPTED {RESET}{GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━┓{RESET}"
        );
        println!(
            "   {GRAY}┃{RESET} {GRAY}TARGET :{RESET} {WHITE}{BOLD}{}{RESET}",
            request.relative_path
        );
        println!(
            "   {GRAY}┃{RESET} {GRAY}SIZE   :{RESET} {SILVER}{} bytes ({} lines){RESET}",
            request.content.len(),
            request.content.lines().count()
        );
        println!("   {GRAY}┠───────────────────────────────────────────────────┨{RESET}");

        let preview: String = request.content.chars().take(600).collect();
        for (i, line) in preview.lines().take(12).enumerate() {
            let clean_line: String = line.chars().take(44).collect();
            println!(
                "   {GRAY}┃ {:02} │{RESET} {SILVER}{clean_line}{RESET}",
                i + 1
            );
        }
        if request.content.chars().count() > 600 || request.content.lines().count() > 12 {
            println!("   {GRAY}┃ .. │ ... [preview truncated]{RESET}");
        }
        println!("   {GRAY}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛{RESET}");
        print!(
            "   {INV_WHITE} CONFIRM {RESET} {WHITE}Authorize disk write?{RESET} {GRAY}[y/N]{RESET} ▪ "
        );
        io::stdout().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        println!();

        Ok(answer.trim().eq_ignore_ascii_case("y"))
    }
}
