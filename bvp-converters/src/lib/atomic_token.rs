use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct AtomicBoolToken {
    token: Arc<AtomicBool>,
}

impl AtomicBoolToken {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            token: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_true(&self) -> bool {
        self.token.load(Ordering::SeqCst)
    }

    pub fn set_false(&self) {
        self.token.store(false, Ordering::SeqCst);
    }

    pub fn set_true(&self) {
        self.token.store(true, Ordering::SeqCst);
    }
}
