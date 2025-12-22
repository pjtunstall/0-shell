use std::sync::atomic::AtomicI32;

// Store the PID of the currently running foreground process.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);
