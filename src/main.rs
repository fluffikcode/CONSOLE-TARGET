//! TARGET binary entry point.
//!
//! This file only wires the layers together: parse arguments, validate
//! settings, build the agent, and run the interactive read-eval-print loop.

use std::{
    io::{self, Write},
    sync::Arc,
};

use anyhow::Result;
use clap::Parser;

use target::agent::Agent;
use target::cli::Args;
use target::config::Settings;
use target::ui::{self, TerminalApprover};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let settings = Settings::from_args(&args)?;
    let mut agent = Agent::new(&settings, Arc::new(TerminalApprover))?;

    ui::print_banner();
    ui::print_dashboard(
        &settings.workdir,
        &settings.model,
        &settings.base_url,
        settings.api_key.is_some(),
    );

    loop {
        ui::print_prompt(agent.message_count())?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;
        print!("{}", ui::RESET);
        io::stdout().flush()?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let task = input.trim();
        if task.is_empty() {
            continue;
        }
        if task.eq_ignore_ascii_case("/exit") || task.eq_ignore_ascii_case("exit") {
            ui::print_system_notice("SESSION TERMINATED.");
            break;
        }
        if task.eq_ignore_ascii_case("/clear") || task.eq_ignore_ascii_case("clear") {
            ui::clear_screen();
            ui::print_banner();
            continue;
        }
        if task.eq_ignore_ascii_case("/reset") || task.eq_ignore_ascii_case("reset") {
            agent.reset_memory();
            ui::print_system_notice("CONTEXT MEMORY PURGED.");
            continue;
        }

        match agent.run_task(task).await {
            Ok(answer) => ui::print_answer(&answer),
            Err(error) => ui::print_error(&error),
        }
    }

    Ok(())
}
