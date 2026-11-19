//! Background spinner shown while waiting for the model.

use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use super::theme::*;

const FRAME_DELAY: Duration = Duration::from_millis(65);

/// A terminal spinner that stops and restores the cursor when dropped.
pub struct Spinner {
    stop_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// Start animating with an uppercased status message.
    pub fn start(message: &str) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&stop_flag);
        let msg = message.to_uppercase();

        let handle = thread::spawn(move || {
            let frames = [
                "█▒▒▒▒",
                "▒█▒▒▒",
                "▒▒█▒▒",
                "▒▒▒█▒",
                "▒▒▒▒█",
                "▒▒▒█▒",
                "▒▒█▒▒",
                "▒█▒▒▒",
            ];
            let start = Instant::now();
            let mut idx = 0;

            print!("\x1b[?25l");
            let _ = io::stdout().flush();

            while !flag_clone.load(Ordering::Relaxed) {
                let elapsed = start.elapsed().as_secs_f32();
                let wave = frames[idx % frames.len()];
                print!(
                    "\r   {GRAY}[{WHITE}{wave}{GRAY}]{RESET} {SILVER}{msg}{RESET} {GRAY}· {:04.1}s{RESET}\x1b[K",
                    elapsed
                );
                let _ = io::stdout().flush();
                idx += 1;
                thread::sleep(FRAME_DELAY);
            }
        });

        Self {
            stop_flag,
            handle: Some(handle),
        }
    }

    /// Stop the animation and wait for the worker thread to exit.
    pub fn stop(mut self) {
        self.cleanup();
    }

    fn cleanup(&mut self) {
        if !self.stop_flag.swap(true, Ordering::Relaxed) {
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
            print!("\r\x1b[2K\x1b[?25h");
            let _ = io::stdout().flush();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.cleanup();
    }
}
