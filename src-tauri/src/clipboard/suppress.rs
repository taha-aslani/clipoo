use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static SUPPRESS_NEXT_CAPTURE: AtomicBool = AtomicBool::new(false);
static SKIP_SEQUENCE: AtomicU32 = AtomicU32::new(0);

pub fn suppress_next_capture() {
    SUPPRESS_NEXT_CAPTURE.store(true, Ordering::SeqCst);
}

pub fn take_suppress_next_capture() -> bool {
    SUPPRESS_NEXT_CAPTURE.swap(false, Ordering::SeqCst)
}

pub fn skip_clipboard_sequence(sequence: u32) {
    SKIP_SEQUENCE.store(sequence, Ordering::SeqCst);
}

pub fn take_skipped_sequence(sequence: u32) -> bool {
    let skipped = SKIP_SEQUENCE.load(Ordering::SeqCst);
    if skipped != 0 && skipped == sequence {
        SKIP_SEQUENCE.store(0, Ordering::SeqCst);
        true
    } else {
        false
    }
}
