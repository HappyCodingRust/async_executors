use crate::{
    BlockOn, InnerJh, JoinHandle, LocalSpawnHandle, LocalSpawnHandleStatic, LocalSpawnStatic,
    SpawnHandle, SpawnHandleStatic, SpawnStatic, YieldNowStatic,
};
use futures_task::{FutureObj, LocalSpawn, Spawn, SpawnError};
use futures_util::future::{BoxFuture, LocalFutureObj};
use futures_util::FutureExt;
use glommio_crate::{LocalExecutor, LocalExecutorBuilder, Task};
use std::future::Future;
use std::rc::Rc;

/// A simple glommio runtime builder
#[derive(Debug)]
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
    fn spawn_local<Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future + 'static,
        Fut::Output: 'static,
    {
        glommio_crate::Task::local(future).detach();
        Ok(())
    }
}
impl<Out: Send + 'static> LocalSpawnHandle<Out> for GlommioCt {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        let (remote, remote_handle) = future.remote_handle();
        let _task = glommio_crate::Task::local(remote).detach();
        Ok(JoinHandle {
            inner: InnerJh::RemoteHandle(Some(remote_handle)),
        })
    }
}
impl LocalSpawnHandleStatic for GlommioCt {
    fn spawn_handle_local<Fut>(
        future: Fut,
    ) -> Result<JoinHandle<<Fut as Future>::Output>, SpawnError>
    where
        Fut: Future + 'static,
        Fut::Output: 'static,
    {
        let (remote, remote_handle) = future.remote_handle();
        let _task = glommio_crate::Task::local(remote).detach();
        Ok(JoinHandle {
            inner: InnerJh::RemoteHandle(Some(remote_handle)),
        })
    }
}
impl Spawn for GlommioCt {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.spawn_local_obj(LocalFutureObj::from(future))
    }
}
impl SpawnStatic for GlommioCt {
    fn spawn<Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
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
    fn spawn_handle<Fut>(future: Fut) -> Result<JoinHandle<<Fut as Future>::Output>, SpawnError>
    where
        Fut: Future + Send + 'static,
        Fut::Output: 'static + Send,
    {
        let (remote, remote_handle) = future.remote_handle();
        let _task = glommio_crate::Task::local(remote).detach();
        Ok(JoinHandle {
            inner: InnerJh::RemoteHandle(Some(remote_handle)),
        })
    }
}
impl YieldNowStatic for GlommioCt {
    fn yield_now() -> BoxFuture<'static, ()> {
        Box::pin(Task::<()>::yield_if_needed())
    }
}
