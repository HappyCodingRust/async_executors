use crate::{AsyncJoinHandle, LocalSpawn, Spawn, SpawnError};
use futures_util::future::{AbortHandle, Aborted};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use {
    crate::{JoinHandle, LocalSpawnHandle, SpawnHandle},
    futures_task::{FutureObj, LocalFutureObj},
    futures_util::future::abortable,
};

/// An executor that spawns tasks on async-std. In contrast to the other executors, this one
/// is not self contained, because async-std does not provide an API that allows that,
/// so the threadpool is global.
///
/// It works on Wasm.
//
#[derive(Copy, Clone, Default)]
//
#[cfg_attr(nightly, doc(cfg(feature = "async_std")))]
//
pub struct AsyncStd;

impl AsyncStd {
    /// Create a new AsyncStd wrapper, forwards to `Default::default`.
    ///
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrapper around [async_std::task::block_on](::async_std_crate::task::block_on()). This is not available on Wasm
    /// as Wasm does not have threads and you're not allowed to block the only thread you have.
    //
    #[cfg(not(target_os = "unknown"))]
    #[cfg_attr(nightly, doc(cfg(not(target_os = "unknown"))))]
    //
    pub fn block_on<F: Future>(future: F) -> F::Output {
        async_std_crate::task::block_on(future)
    }
}

#[cfg(target_arch = "wasm32")]
//
impl Spawn for AsyncStd {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        async_std_crate::task::spawn_local(future);

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
//
impl Spawn for AsyncStd {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        async_std_crate::task::spawn(future);

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
//
impl<Out: 'static + Send> SpawnHandle<Out> for AsyncStd {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        let (fut, a_handle) = abortable(future);

        Ok(AsyncStdJoinHandle::new(async_std_crate::task::spawn(fut), a_handle).into())
    }
}

#[cfg(target_arch = "wasm32")]
//
impl<Out: 'static + Send> SpawnHandle<Out> for AsyncStd {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        let (fut, a_handle) = abortable(future);

        Ok(AsyncStdJoinHandle::new(async_std_crate::task::spawn_local(fut), a_handle).into())
    }
}

impl<Out: 'static> LocalSpawnHandle<Out> for AsyncStd {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        let (fut, a_handle) = abortable(future);

        Ok(AsyncStdJoinHandle::new(async_std_crate::task::spawn_local(fut), a_handle).into())
    }
}

impl LocalSpawn for AsyncStd {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        // We drop the JoinHandle, so the task becomes detached.
        //
        let _ = async_std_crate::task::spawn_local(future);

        Ok(())
    }
}

impl std::fmt::Debug for AsyncStd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncStd executor")
    }
}

#[derive(Debug)]
pub struct AsyncStdJoinHandle<T> {
    task: Option<async_std_crate::task::JoinHandle<Result<T, Aborted>>>,
    a_handle: AbortHandle,
}
impl<T> AsyncStdJoinHandle<T> {
    pub fn new(
        task: async_std_crate::task::JoinHandle<Result<T, Aborted>>,
        a_handle: AbortHandle,
    ) -> Self {
        Self {
            task: Some(task),
            a_handle,
        }
    }
}
impl<T> Future for AsyncStdJoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match futures_util::ready!(Pin::new(
            self.task
                .as_mut()
                .expect("Cannot poll a detached JoinHandle twice")
        )
        .poll(cx))
        {
            Ok(x) => Poll::Ready(x),
            Err(_) => {
                panic!("Task has been aborted")
            }
        }
    }
}
impl<T> AsyncJoinHandle for AsyncStdJoinHandle<T> {
    fn detach(mut self)
    where
        Self: Sized,
    {
        self.task.take();
    }
}
impl<T> Drop for AsyncStdJoinHandle<T> {
    fn drop(&mut self) {
        if self.task.is_some() {
            self.a_handle.abort();
        }
    }
}
impl<T> Into<JoinHandle<T>> for AsyncStdJoinHandle<T> {
    fn into(self) -> JoinHandle<T> {
        JoinHandle::AsyncStdJoinHandle(self)
    }
}
