//! Functions in this module run in a Hyperlight VM and do the following:
//! 1. Dispatch a guest function call (i.e., `PrintOutput`) to the Hyperlight VM.
//! 2. Because the guest function does not have access to any syscalls due to it
//! running with no OS, to print to the console, it must call a host function—which
//! is done via a VMExit. Upon completion of the host function, we re-enter the VM.
//! Then, the VM can obtain the result from the host function call from shared memory.
//! 3. Finally, exit the VM and return the result to the client.

use warp::Filter;

use hyperlight_host::{Sandbox, UninitializedSandbox};

use crate::hyperlight::{acquire_sandbox, release_sandbox, SandboxError};

pub(crate) fn cold() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("hyperlight")
        .and(warp::path("hello-world"))
        .and(warp::path("cold"))
        .map(move || {
            // Create a new sandbox configuration
            let cfg = {
                #[cfg(feature = "gdb")]
                {
                    use hyperlight_host::sandbox::{config::DebugInfo, SandboxConfiguration};

                    let mut cfg = SandboxConfiguration::default();
                    let debug = DebugInfo { port: 8080 };
                    cfg.set_guest_debug_info(debug);

                    Some(cfg)
                }

                #[cfg(not(feature = "gdb"))]
                {
                    // Default configuration without GDB
                    None
                }
            };

            // Create a new sandbox per request
            let uninitialized_sandbox = UninitializedSandbox::new(
                hyperlight_host::GuestBinary::FilePath(crate::DEMO_GUEST_PATH.into()),
                cfg,  // configuration
            )
            .unwrap();
            let mut multi_use_sandbox: Sandbox =
                uninitialized_sandbox.evolve().unwrap();

            let message = "Hello, World! I am executing inside of a VM :)\n".to_string();
            multi_use_sandbox
                .call::<i32>(
                    "PrintOutput",
                    message.clone(),
                )
                .unwrap();

            "Guest function called inside cold Hyperlight VM.".to_string()
        })
}

pub(crate) fn warm() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("hyperlight")
        .and(warp::path("hello-world"))
        .and(warp::path("warm"))
        .and_then(move || async {
            let message = "Hello, World! I am executing inside of a VM :)\n".to_string();

            // Acquire a sandbox (create if necessary, reuse if available)
            let sandbox = acquire_sandbox().await;

            // Lock and use the selected sandbox
            {
                let mut sandbox_guard = sandbox.lock().await;
                sandbox_guard
                    .call::<i32>(
                        "PrintOutput",
                        message.clone(),
                    )
                    .map_err(|_| warp::reject::custom(SandboxError))?;
                // `sandbox_guard` goes out of scope here and is dropped
            }

            // Release the sandbox back to the pool
            release_sandbox(sandbox).await;

            Ok::<_, warp::Rejection>("Guest function called inside warm Hyperlight VM.".to_string())
        })
}
