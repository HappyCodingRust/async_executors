use crate::{JoinHandle, SpawnError, StaticRuntime};

/// Spawn a blocking task, maybe in a thread pool(tokio), or in current thread and spawns a new thread(std-async)
pub trait SpawnBlocking<T: Send + 'static> {
    /// spawn a blocking function
    fn spawn_blocking_obj(
        &self,
        func: Box<dyn FnOnce() -> T + Send>,
    ) -> Result<JoinHandle<T>, SpawnError>;
}

/// Spawn a blocking task, maybe in a thread pool(tokio), or in current thread and spawns a new thread(std-async)
pub trait SpawnBlockingExt<T: Send + 'static>: SpawnBlocking<T> {
    /// spawn a blocking function
    fn spawn_blocking(
        &self,
        func: impl FnOnce() -> T + Send + 'static,
    ) -> Result<JoinHandle<T>, SpawnError> {
        self.spawn_blocking_obj(Box::new(func))
    }
}

/// Spawn a blocking task, maybe in a thread pool(tokio), or in current thread and spawns a new thread(std-async)
pub trait SpawnBlockingStatic: StaticRuntime {
    /// spawn a blocking function
    fn spawn_blocking<T: Send + 'static>(
        func: impl FnOnce() -> T + Send + 'static,
    ) -> Result<JoinHandle<T>, SpawnError>;
}
