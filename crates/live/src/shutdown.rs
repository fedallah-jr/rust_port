//! SIGINT / SIGTERM shutdown wiring.
//!
//! `signal_hook::flag::register` flips an `AtomicBool` to **true** on signal
//! arrival; the engine loop polls it and exits when set. Two signals are
//! handled:
//!   - `SIGINT` for interactive Ctrl-C
//!   - `SIGTERM` so Docker / systemd `stop` triggers the same clean save
//!
//! After the first signal arms the flag, a second signal forces an immediate
//! process exit — guards against the loop being wedged inside a slow API
//! call. This is enabled via `register_conditional_shutdown`.
//!
//! Tests bypass this entirely: `LiveEngine::start_for_test` skips the
//! signal-handler installation so `cargo test` doesn't hijack SIGINT.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;

/// Register both `SIGINT` and `SIGTERM` as triggers that flip
/// `shutdown_requested` to `true`. After the first signal, a second one of
/// the same kind force-exits the process to break out of stuck syscalls.
///
/// **Registration order matters.** signal-hook runs handlers in the order
/// they were registered; for the conditional shutdown to behave correctly,
/// it must be registered *before* the flag-arming handler:
///   - First handler (conditional): sees flag=false on the first signal,
///     does nothing; sees flag=true on the second signal, terminates.
///   - Second handler (flag): arms the flag.
///
/// If we registered them the other way around, the first signal would arm
/// the flag *before* the conditional handler ran, the conditional would
/// see flag=true, and the process would terminate immediately — bypassing
/// the engine's final save + teardown. See signal-hook docs for
/// `register_conditional_shutdown`.
pub fn install(shutdown_requested: &Arc<AtomicBool>) -> std::io::Result<()> {
    // Register the conditional-shutdown handlers FIRST.
    flag::register_conditional_shutdown(SIGINT, 130, shutdown_requested.clone())?;
    flag::register_conditional_shutdown(SIGTERM, 143, shutdown_requested.clone())?;
    // Then the flag-arming handlers.
    flag::register(SIGINT, shutdown_requested.clone())?;
    flag::register(SIGTERM, shutdown_requested.clone())?;
    Ok(())
}
