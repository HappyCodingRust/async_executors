#[allow(unused_imports)] // some imports are conditional on features
//
use {
    futures_util::{
        future::{AbortHandle, Aborted, RemoteHandle},
        ready,
    },
    std::{
        future::Future,
        sync::atomic::{AtomicBool, Ordering},
    },
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
};

/// A Join Handle that can join both async tasks and blocking tasks
pub trait AsyncJoinHandle: Future {
    /// Drops this handle without canceling the underlying future.
    ///
    /// This method can be used if you want to drop the handle, but let the execution continue.
    fn detach(self)
    where
        Self: Sized;
}

impl<T: 'static> AsyncJoinHandle for RemoteHandle<T> {
    fn detach(self)
    where
        Self: Sized,
    {
        self.forget()
    }
}
impl<T> From<RemoteHandle<T>> for JoinHandle<T> {
    fn from(x: RemoteHandle<T>) -> Self {
        Self::RemoteHandle(x)
    }
}

/// A framework agnostic JoinHandle type. Cancels the future on dropping the handle.
/// You can call [`detach`](JoinHandle::detach) to leave the future running when dropping the handle.
///
/// This leverages the performance gains from the native join handles compared to
/// [RemoteHandle](futures_util::future::RemoteHandle) where possible.
///
/// It does wrap futures in [Abortable](futures_util::future::Abortable) where needed as
/// [tokio] and [async-std](async_std_crate) don't support canceling out of the box.
///
/// # Panics
///
/// There is an inconsistency between executors when it comes to a panicking task.
/// Generally we unwind the thread on which the handle is awaited when a task panics,
/// but async-std will also let the executor thread unwind. No `catch_unwind` was added to
/// bring async-std in line with the other executors here.
///
/// Awaiting the JoinHandle can also panic if you drop the executor before it completes.
#[must_use = "JoinHandle will cancel your future when dropped."]
#[derive(Debug)]
pub enum JoinHandle<T> {
    RemoteHandle(RemoteHandle<T>),
    #[cfg(feature = "tokio")]
    TokioJoinHandle(crate::TokioJoinHandle<T>),
    #[cfg(feature = "async_global")]
    AsyncJoinHandle(crate::AsyncGlobalJoinHandle<T>),
    #[cfg(feature = "async_std")]
    AsyncStdJoinHandle(crate::AsyncStdJoinHandle<T>),
}
impl<T> Unpin for JoinHandle<T> {}
impl<T: 'static> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut *self {
            JoinHandle::RemoteHandle(x) => Pin::new(x).poll(cx),
            #[cfg(feature = "tokio")]
            JoinHandle::TokioJoinHandle(x) => Pin::new(x).poll(cx),
            #[cfg(feature = "async_global")]
            JoinHandle::AsyncJoinHandle(x) => Pin::new(x).poll(cx),
            #[cfg(feature = "async_std")]
            JoinHandle::AsyncStdJoinHandle(x) => Pin::new(x).poll(cx),
        }
    }
}
impl<T: 'static> AsyncJoinHandle for JoinHandle<T> {
    fn detach(self)
    where
        Self: Sized,
    {
        match self {
            JoinHandle::RemoteHandle(x) => x.detach(),
            #[cfg(feature = "tokio")]
            JoinHandle::TokioJoinHandle(x) => x.detach(),
            #[cfg(feature = "async_global")]
            JoinHandle::AsyncJoinHandle(x) => x.detach(),
            #[cfg(feature = "async_std")]
            JoinHandle::AsyncStdJoinHandle(x) => x.detach(),
        }
    }
}
#[cfg(test)]
//
mod tests {
    use super::*;

    // It's important that this is not Send, as we allow spawning !Send futures on it.
    //
    static_assertions::assert_impl_all!(JoinHandle<()>: Send);
}
