use crate::{AsyncJoinHandle, LocalSpawn, Spawn, SpawnError};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use {
    crate::{JoinHandle, LocalSpawnHandle, SpawnHandle},
    futures_task::{FutureObj, LocalFutureObj},
};

/// An executor that spawns tasks on async-global-executor. In contrast to the other executors, this one
/// is not self contained, because async-global-executor does not provide an API that allows that,
/// so the threadpool is global.
///
/// It works on Wasm.
//
#[derive(Copy, Clone, Default)]
//
#[cfg_attr(nightly, doc(cfg(feature = "async_global")))]
//
pub struct AsyncGlobal;

impl AsyncGlobal {
    /// Create a new AsyncGlobal wrapper, forwards to `Default::default`.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrapper around [async_global_executor::block_on]. This is not available on Wasm
    /// as Wasm does not have threads and you're not allowed to block the only thread you have.
    //
    // TODO: is target_arch = "wasm32"  not a better way to express this?
    //
    #[cfg(not(target_os = "unknown"))]
    #[cfg_attr(nightly, doc(cfg(not(target_os = "unknown"))))]
    //
    pub fn block_on<F: Future>(future: F) -> F::Output {
        async_global_executor::block_on(future)
    }
}

#[cfg(target_arch = "wasm32")]
//
impl Spawn for AsyncGlobal {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        async_global_executor::spawn_local(future).detach();

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
//
impl Spawn for AsyncGlobal {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        async_global_executor::spawn(future).detach();

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
//
impl<Out: 'static + Send> SpawnHandle<Out> for AsyncGlobal {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Ok(AsyncGlobalJoinHandle::new(async_global_executor::spawn(future)).into())
    }
}

#[cfg(target_arch = "wasm32")]
//
impl<Out: 'static + Send> SpawnHandle<Out> for AsyncGlobal {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Ok(AsyncGlobalJoinHandle::new(async_global_executor::spawn_local(future)).into())
    }
}

impl<Out: 'static> LocalSpawnHandle<Out> for AsyncGlobal {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Ok(AsyncGlobalJoinHandle::new(async_global_executor::spawn_local(future)).into())
    }
}

impl LocalSpawn for AsyncGlobal {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        let _ = async_global_executor::spawn_local(future).detach();

        Ok(())
    }
}

impl std::fmt::Debug for AsyncGlobal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncGlobal executor")
    }
}
#[derive(Debug)]
pub struct AsyncGlobalJoinHandle<T>(async_global_executor::Task<T>);
impl<T> AsyncGlobalJoinHandle<T> {
    pub fn new(task: async_global_executor::Task<T>) -> Self {
        Self(task)
    }
}
impl<T> Future for AsyncGlobalJoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}
impl<T> AsyncJoinHandle for AsyncGlobalJoinHandle<T> {
    fn detach(self)
    where
        Self: Sized,
    {
        self.0.detach()
    }
}
impl<T> Into<JoinHandle<T>> for AsyncGlobalJoinHandle<T> {
    fn into(self) -> JoinHandle<T> {
        JoinHandle::AsyncJoinHandle(self)
    }
}
