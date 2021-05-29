use crate::{
    BlockOn, JoinHandle, LocalSpawn, LocalSpawnHandle, LocalSpawnHandleStatic, LocalSpawnStatic,
    Spawn, SpawnBlockingStatic, SpawnError, SpawnHandle, SpawnHandleStatic, SpawnStatic,
    YieldNowStatic,
};
use futures_task::FutureObj;
use futures_util::future::{BoxFuture, LocalFutureObj};
use futures_util::FutureExt;
use glommio_crate::{LocalExecutor, LocalExecutorBuilder, Task};
use std::future::Future;
use std::rc::Rc;

/// A simple glommio runtime builder
#[derive(Debug, Clone)]
pub struct GlommioCt {
    executor: Rc<LocalExecutor>,
}

impl GlommioCt {
    /// new Glommio Local Executor
    pub fn new(name: &str, cpu_set: Option<usize>) -> Self {
        let mut builder = LocalExecutorBuilder::new().name(&name);
        if let Some(binding) = cpu_set {
            builder = builder.pin_to_cpu(binding);
        }
        let executor = builder.make().unwrap();
        Self {
            executor: Rc::new(executor),
        }
    }
    /// execute the code until completion
    pub fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        self.executor.run(future)
    }
}

impl BlockOn for GlommioCt {
    fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        Self::block_on(self, future)
    }
}

impl LocalSpawn for GlommioCt {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        glommio_crate::Task::local(future).detach();
        Ok(())
    }
}
impl LocalSpawnStatic for GlommioCt {
    fn spawn_local<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
    {
        glommio_crate::Task::local(future).detach();
        Ok(())
    }
}
impl<Out: 'static> LocalSpawnHandle<Out> for GlommioCt {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        GlommioCt::spawn_handle_local(future)
    }
}
impl LocalSpawnHandleStatic for GlommioCt {
    fn spawn_handle_local<Output, Fut>(future: Fut) -> Result<JoinHandle<Output>, SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
    {
        let (remote, handle) = future.remote_handle();
        Task::local(remote).detach();
        Ok(handle.into())
    }
}
impl Spawn for GlommioCt {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.spawn_local_obj(LocalFutureObj::from(future))
    }
}

impl SpawnStatic for GlommioCt {
    fn spawn<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        glommio_crate::Task::local(future).detach();
        Ok(())
    }
}
impl<Out: Send + 'static> SpawnHandle<Out> for GlommioCt {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        <GlommioCt as SpawnHandleStatic>::spawn_handle(future)
    }
}
impl SpawnHandleStatic for GlommioCt {
    fn spawn_handle<Output, Fut>(future: Fut) -> Result<JoinHandle<Output>, SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: 'static + Send,
    {
        let (remote, handle) = future.remote_handle();
        glommio_crate::Task::local(remote).detach();
        Ok(handle.into())
    }
}
impl YieldNowStatic for GlommioCt {
    fn yield_now() -> BoxFuture<'static, ()> {
        Box::pin(Task::<()>::yield_if_needed())
    }
}
impl SpawnBlockingStatic for GlommioCt {
    fn spawn_blocking<T: Send + 'static>(
        func: impl FnOnce() -> T + Send + 'static,
    ) -> Result<JoinHandle<T>, SpawnError> {
        let (remote, handle) = async { func() }.remote_handle();
        std::thread::spawn(move || futures_executor::block_on(remote));
        Ok(handle.into())
    }
}
#[cfg(test)]
//
mod tests {
    use super::*;

    // It's important that this is not Send, as we allow spawning !Send futures on it.
    //
    static_assertions::assert_not_impl_any!(GlommioCt: Send, Sync);
}
