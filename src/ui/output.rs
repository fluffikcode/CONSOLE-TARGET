//! Print helpers for prompts, tool activity, answers, and errors.

use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use super::theme::*;

/// Prompt header showing the conversation depth.
pub fn print_prompt(msg_count: usize) -> anyhow::Result<()> {
    println!(
        "   {GRAY}╭─[{RESET} {WHITE}{BOLD}target{RESET} {GRAY}· ctx:{:02} ]{RESET}",
        msg_count.saturating_sub(1)
    );
    print!("   {GRAY}╰─▪{RESET} {WHITE}");
    io::stdout().flush()?;
    Ok(())
}

/// System-level notice, such as a context reset.
pub fn print_system_notice(msg: &str) {
    println!("\n   {INV_GRAY} SYS {RESET} {SILVER}{msg}{RESET}\n");
}

/// Header for a tool call, including a truncated argument preview.
pub fn print_tool_call(step: usize, max_steps: usize, name: &str, args: &str) {
    let clean_args: String = args.chars().take(52).collect();
    let ellipsis = if args.chars().count() > 52 { "..." } else { "" };

    println!(
        "   {GRAY}╭─{RESET}{INV_WHITE} STEP {:02}/{:02} {RESET}{GRAY}─[{RESET} {WHITE}{BOLD}{}{RESET} {GRAY}]{RESET}",
        step,
        max_steps,
        name.to_uppercase()
    );
    println!("   {GRAY}│ ▸ args :{RESET} {GRAY}{clean_args}{ellipsis}{RESET}");
}

/// Result of a tool call, with a short preview for multi-line output.
pub fn print_tool_result(result: &str, is_err: bool) {
    let lines: Vec<&str> = result.lines().collect();
    let total = lines.len();
    let preview_max = 5;

    if is_err {
        println!(
            "   {GRAY}╰─▪{RESET} {INV_GRAY} ERR {RESET} {WHITE}{}{RESET}\n",
            result
        );
        return;
    }

    if total <= 1 {
        println!(
            "   {GRAY}╰─▪{RESET} {WHITE}{BOLD}OK{RESET}   {GRAY}:{RESET} {SILVER}{}{RESET}\n",
            result.trim()
        );
    } else {
        println!(
            "   {GRAY}╰─▪{RESET} {WHITE}{BOLD}OK{RESET}   {GRAY}: {SILVER}{total} lines returned{RESET}"
        );
        for (idx, line) in lines.iter().take(preview_max).enumerate() {
            let truncated: String = line.chars().take(64).collect();
            let sfx = if line.chars().count() > 64 { "…" } else { "" };
            println!("       {GRAY}{:02} │ {DIM}{truncated}{sfx}{RESET}", idx + 1);
        }
        if total > preview_max {
            println!(
                "       {GRAY}.. │ +{} more lines in context{RESET}",
                total - preview_max
            );
        }
        println!();
    }
}

/// Final model answer inside a framed panel, typed out character by character.
pub fn print_answer(answer: &str) {
    println!(
        "   {GRAY}┏━━━{RESET}{INV_WHITE} ❖ TARGET {RESET}{GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓{RESET}"
    );
    for line in answer.lines() {
        print!("   {GRAY}┃{RESET} ");
        for ch in line.chars() {
            print!("{WHITE}{ch}{RESET}");
            let _ = io::stdout().flush();
            thread::sleep(Duration::from_millis(1));
        }
        println!();
    }
    println!("   {GRAY}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛{RESET}\n");
}

/// Fatal or recoverable error rendered as a framed exception panel.
pub fn print_error(err: &anyhow::Error) {
    eprintln!(
        "\n   {GRAY}┏━━━{RESET}{INV_WHITE} ✗ EXCEPTION {RESET}{GRAY}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓{RESET}"
    );
    for line in format!("{:#}", err).lines() {
        eprintln!("   {GRAY}┃{RESET} {SILVER}{line}{RESET}");
    }
    eprintln!("   {GRAY}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛{RESET}\n");
}
