use hyperlight_host::{Sandbox, UninitializedSandbox};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::sync::atomic::{AtomicUsize, Ordering};
use warp::Filter;

pub(super) mod hello_world;
pub(super) mod safety;

const INITIAL_POOL_SIZE: usize = 10;
const MAX_POOL_SIZE: usize = 100;
static CREATED_VM_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct SandboxError;
impl warp::reject::Reject for SandboxError {}

pub(crate) static SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(MAX_POOL_SIZE)));

// Pool of sandboxes, initialized lazily
pub(crate) static MULTI_USE_SANDBOX_POOL: Lazy<Arc<Mutex<Vec<Arc<Mutex<Sandbox>>>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Vec::with_capacity(MAX_POOL_SIZE)))
});

// Helper function to create a new sandbox
async fn create_sandbox() -> Arc<Mutex<Sandbox>> {
    // Increment the counter before creating a new sandbox
    CREATED_VM_COUNT.fetch_add(1, Ordering::SeqCst);

    let uninitialized_sandbox = UninitializedSandbox::new(
        hyperlight_host::GuestBinary::FilePath(crate::DEMO_GUEST_PATH.into()),
        None, // default configuration
    ).unwrap();

    let sandbox = uninitialized_sandbox.evolve().unwrap();

    Arc::new(Mutex::new(
        sandbox,
    ))
}


// Function to acquire a sandbox, creating a new one if the pool is empty
pub(super) async fn acquire_sandbox() -> Arc<Mutex<Sandbox>> {
    // Wait for an available permit, respecting the max pool size
    let _permit = SEMAPHORE.acquire().await.unwrap();

    // Try to get an existing sandbox from the pool
    let mut pool = MULTI_USE_SANDBOX_POOL.lock().await;
    if let Some(sandbox) = pool.pop() {
        sandbox
    } else {
        // Create a new sandbox if the pool is empty and return it
        create_sandbox().await
    }
}

// Function to release a sandbox back to the pool
pub(super) async fn release_sandbox(sandbox: Arc<Mutex<Sandbox>>) {
    let mut pool = MULTI_USE_SANDBOX_POOL.lock().await;
    if pool.len() < MAX_POOL_SIZE {
        pool.push(sandbox); // Return the sandbox to the pool if under the max limit
    }
}

// Function to warm up the initial pool with a few instances
pub(super) async fn warm_up_pool() {
    let mut pool = MULTI_USE_SANDBOX_POOL.lock().await;
    for _ in 0..INITIAL_POOL_SIZE {
        let sandbox = create_sandbox().await;
        pool.push(sandbox);
        println!("Sandbox instance warmed up.");
    }
}

pub(crate) fn get_vm_count() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    warp::path("hyperlight")
        .and(warp::path("vm-count"))
        .map(|| {
            let count = CREATED_VM_COUNT.load(Ordering::SeqCst);
            format!("Number of created VMs: {}", count)
        })
}
