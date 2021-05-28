use crate::{
    BlockOn, InnerJh, JoinHandle, LocalSpawnHandle, LocalSpawnHandleStatic, LocalSpawnStatic,
    SpawnHandle, SpawnHandleExt, SpawnHandleStatic, SpawnStatic, YieldNowStatic,
};
use futures_task::{FutureObj, LocalSpawn, Spawn, SpawnError};
use futures_util::future::{BoxFuture, LocalFutureObj};
use futures_util::FutureExt;
use glommio_crate::{LocalExecutorBuilder, Task};
use std::future::Future;

/// A simple glommio runtime builder
#[derive(Debug)]
pub struct GlommioCtBuilder {
    binding: Option<usize>,
    name: String,
}

impl GlommioCtBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            binding: None,
            name: "unnamed".to_string(),
        }
    }

    /// Sets the new executor's affinity to the provided CPU.  The largest `cpu`
    /// value [supported] by libc is 1023.
    ///
    /// [supported]: https://man7.org/linux/man-pages/man2/sched_setaffinity.2.html#NOTES
    pub fn pin_to_cpu(&mut self, cpu: usize) {
        self.binding = Some(cpu);
    }

    fn get_builder(&self) -> LocalExecutorBuilder {
        let mut builder = LocalExecutorBuilder::new().name(&self.name);
        if let Some(binding) = self.binding {
            builder = builder.pin_to_cpu(binding);
        }
        builder
    }
    /// Run current future until completion
    pub fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        self.get_builder()
            .make()
            .expect("Cannot make a local executor")
            .run(future)
    }
}
impl BlockOn for GlommioCtBuilder {
    fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        Self::block_on(self, future)
    }
}
impl Spawn for GlommioCtBuilder {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        GlommioCt::new().spawn_obj(future)
    }
}

impl<Out: Send + 'static> SpawnHandle<Out> for GlommioCtBuilder {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        GlommioCt::new().spawn_handle(future)
    }
}
impl LocalSpawn for GlommioCtBuilder {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        GlommioCt::new().spawn_local_obj(future)
    }
}

impl<Out: Send + 'static> LocalSpawnHandle<Out> for GlommioCtBuilder {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        GlommioCt::new().spawn_handle_local_obj(future)
    }
}

/// Glommio Local Executor
#[derive(Debug, Copy, Clone)]
pub struct GlommioCt {}

impl GlommioCt {
    /// new Glommio Local Executor
    pub fn new() -> Self {
        Self {}
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
