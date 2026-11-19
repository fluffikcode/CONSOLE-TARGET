//! Terminal UI: monochrome theme, banner, dashboard, tool output, and prompts.
//!
//! Rendering is deliberately isolated from the agent loop. The only UI
//! dependency of the agent is the [`Spinner`] shown while waiting for the
//! model, plus the print helpers used for tool activity.

mod approval;
mod banner;
mod dashboard;
mod output;
mod spinner;
mod theme;

pub use approval::TerminalApprover;
pub use banner::{clear_screen, print_banner};
pub use dashboard::print_dashboard;
pub use output::{
    print_answer, print_error, print_prompt, print_system_notice, print_tool_call,
    print_tool_result,
};
pub use spinner::Spinner;
pub use theme::{BOLD, DIM, GRAY, INV_GRAY, INV_WHITE, RESET, SILVER, WHITE};
