use crate::{
    JoinHandle, LocalSpawnHandleStatic, LocalSpawnStatic, SpawnBlockingStatic, SpawnError,
    SpawnHandleStatic, SpawnStatic, TokioJoinHandle, YieldNowStatic,
};
use futures_util::future::BoxFuture;
use std::future::Future;

#[derive(Debug, Copy, Clone)]
struct Tokio;

impl SpawnStatic for Tokio {
    fn spawn<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        let _ = tokio::task::spawn(future);
        Ok(())
    }
}

impl LocalSpawnStatic for Tokio {
    fn spawn_local<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
    {
        let _ = tokio::task::spawn_local(future);
        Ok(())
    }
}

impl SpawnHandleStatic for Tokio {
    fn spawn_handle<Output, Fut>(future: Fut) -> Result<JoinHandle<Output>, SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: 'static + Send,
    {
        Ok(TokioJoinHandle::new(tokio::task::spawn(future)).into())
    }
}

impl LocalSpawnHandleStatic for Tokio {
    fn spawn_handle_local<Output, Fut>(future: Fut) -> Result<JoinHandle<Output>, SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
    {
        Ok(TokioJoinHandle::new(tokio::task::spawn_local(future)).into())
    }
}

impl YieldNowStatic for Tokio {
    fn yield_now() -> BoxFuture<'static, ()> {
        Box::pin(tokio::task::yield_now())
    }
}

impl SpawnBlockingStatic for Tokio {
    fn spawn_blocking<T: Send + 'static>(
        func: impl FnOnce() -> T + Send + 'static,
    ) -> Result<JoinHandle<T>, SpawnError> {
        let handle = tokio::task::spawn_blocking(func);
        Ok(TokioJoinHandle::new(handle).into())
    }
}
