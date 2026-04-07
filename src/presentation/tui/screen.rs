use console::Term;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

static EXIT_MESSAGE_PRINTED: AtomicBool = AtomicBool::new(false);

/// RAII guard that activates the terminal alternate screen and restores it on drop.
pub struct AlternateScreenGuard {
    pub term: Term,
    pub is_active: bool,
}

impl AlternateScreenGuard {
    pub fn new(term: Term) -> Result<Self> {
        let _ = term.write_str("\x1b[?1049h");
        let _ = term.hide_cursor();

        EXIT_MESSAGE_PRINTED.store(false, Ordering::SeqCst);

        let t = term.clone();
        let _ = ctrlc::set_handler(move || {
            let _ = t.write_str("\x1b[?1049l");
            let _ = t.show_cursor();
            if !EXIT_MESSAGE_PRINTED.swap(true, Ordering::SeqCst) {
                let _ = t.write_line("USAGI run away ( ^-^)ノ");
            }
            std::process::exit(0);
        });

        Ok(Self { term, is_active: true })
    }

    pub fn dismiss(&mut self) {
        self.is_active = false;
    }
}

impl Drop for AlternateScreenGuard {
    fn drop(&mut self) {
        let _ = self.term.write_str("\x1b[?1049l");
        let _ = self.term.show_cursor();
        if self.is_active {
            if !EXIT_MESSAGE_PRINTED.swap(true, Ordering::SeqCst) {
                let _ = self.term.write_line("USAGI run away ( ^-^)ノ");
            }
        }
    }
}
