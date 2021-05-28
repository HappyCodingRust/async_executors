/// reexport futures_task::Spawn etc
pub use futures_task::{LocalSpawn, Spawn};

use futures_task::SpawnError;
use std::future::Future;

/// The `SpawnStatic` trait allows for pushing futures onto an executor that will
/// run them to completion. Except that this is used for ZST as type
pub trait SpawnStatic {
    /// Spawns a future that will be run to completion
    fn spawn<Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static;
}

/// The `LocalSpawnStatic` is similar to [`SpawnStatic`], but allows spawning futures
/// that don't implement `Send`.
pub trait LocalSpawnStatic {
    /// Spawns a future that will be run to completion
    fn spawn_local<Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future + 'static,
        Fut::Output: 'static;
}
