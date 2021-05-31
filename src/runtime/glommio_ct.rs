use crate::{
    BlockOn, CoreAffinityGuard, JoinHandle, LocalSpawn, LocalSpawnHandle, Spawn, SpawnError,
    SpawnHandle, SpawnHandleStatic,
};
use crate::{Glommio, LocalSpawnHandleStatic};
use futures_task::FutureObj;
use futures_util::future::LocalFutureObj;
use glommio_crate::{LocalExecutor, LocalExecutorBuilder};
use std::future::Future;
use std::rc::Rc;

/// A simple glommio runtime builder
#[derive(Debug, Clone)]
pub struct GlommioCt {
    guard: Rc<CoreAffinityGuard>,
    executor: Rc<LocalExecutor>,
}

impl GlommioCt {
    /// new Glommio Local Executor
    pub fn new(name: &str, cpu_set: Option<usize>) -> Self {
        let guard = Rc::new(CoreAffinityGuard::new().unwrap());
        let mut builder = LocalExecutorBuilder::new().name(&name);
        if let Some(binding) = cpu_set {
            builder = builder.pin_to_cpu(binding);
        }
        let executor = builder.make().unwrap();
        Self {
            guard,
            executor: Rc::new(executor),
        }
    }
    /// execute the code until completion
    pub fn block_on<F: Future>(&self, future: F) -> <F as Future>::Output {
        let val = self.executor.run(future);
        val
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

impl<Out: 'static> LocalSpawnHandle<Out> for GlommioCt {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Glommio::spawn_handle_local(future)
    }
}

impl Spawn for GlommioCt {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.spawn_local_obj(LocalFutureObj::from(future))
    }
}

impl<Out: Send + 'static> SpawnHandle<Out> for GlommioCt {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Glommio::spawn_handle(future)
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
