use crate::AsyncJoinHandle;
use futures_util::ready;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct TokioJoinHandle<T> {
    handle: Option<JoinHandle<T>>,
}
impl<T> TokioJoinHandle<T> {
    pub fn new(handle: JoinHandle<T>) -> Self {
        Self {
            handle: Some(handle)
        }
    }
}
impl<T> Unpin for TokioJoinHandle<T> {}

impl<T> Future for TokioJoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        if let Some(handle) = &mut this.handle {
            match ready!(Pin::new(handle).poll(cx)) {
                Ok(t) => {
                    this.handle = None;
                    Poll::Ready(t)
                }
                Err(err) => {
                    panic!("Tokio runtime ended before joining: {}", err)
                }
            }
        } else {
            panic!("Cannot poll after completion/cancellation")
        }
    }
}

impl<T> AsyncJoinHandle for TokioJoinHandle<T> {
    fn detach(mut self) {
        self.handle = None;
    }
}
impl<T> Drop for TokioJoinHandle<T> {
    fn drop(&mut self) {
        if let Some(handle) = &mut self.handle {
            handle.abort();
        }
    }
}
impl<T> Into<crate::JoinHandle<T>> for TokioJoinHandle<T> {
    fn into(self) -> crate::JoinHandle<T> {
        crate::JoinHandle::TokioJoinHandle(self)
    }
}