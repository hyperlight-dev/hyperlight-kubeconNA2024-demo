use crate::hyperlight::{acquire_sandbox, release_sandbox};

use warp::Filter;

pub(crate) fn deref_raw_null_ptr() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    warp::path("hyperlight")
        .and(warp::path("safety"))
        .and(warp::path("deref-raw-null-ptr"))
        .and_then(move || async {
            // Acquire a sandbox (create if necessary, reuse if available)
            let sandbox = acquire_sandbox().await;

            // Lock and use the selected sandbox
            let result = {
                let mut sandbox_guard = sandbox.lock().await;
                sandbox_guard.call::<()>(
                    "DereferenceRawNullPointer",
                    (),
                )
            };
            // `sandbox_guard` goes out of scope here and is dropped

            match result {
                Ok(_) => {
                    release_sandbox(sandbox).await;
                    Ok::<_, warp::Rejection>("Guest function called inside warm Hyperlight VM.".to_string())
                },
                // A guest abort poisons the sandbox; do not put it back in the pool.
                Err(err) => Ok(format!("{:?}", err)),
            }
        })
}
