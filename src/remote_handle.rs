use {
    futures_core::ready,
    futures_util::future::{CatchUnwind, FutureExt},
    futures_channel::oneshot::{self, Sender, Receiver},
    pin_utils::{unsafe_pinned, unsafe_unpinned},
    std::{
        any::Any,
        fmt,
        panic::{self, AssertUnwindSafe},
        pin::Pin,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        future::Future,
        task::{ Context, Poll },
    },
};

/// The handle to a remote future returned by
/// [`remote_handle`](crate::future::FutureExt::remote_handle). When you drop this,
/// the remote future will be woken up to be dropped by the executor.
///
/// ## Unwind safety
///
/// When the remote future panics, [Remote] will catch the unwind and transfer it to
/// the thread where `RemoteHandle` is being awaited. This is good for the common
/// case where [Remote] is spawned on a threadpool. It is unlikely that other code
/// in the executor working thread shares mutable data with the spawned future and we
/// preserve the executor from losing its working threads.
///
/// If you run the future locally and send the handle of to be awaited elsewhere, you
/// must be careful with regard to unwind safety because the thread in which the future
/// is polled will keep running after the panic and the thread running the [RemoteHandle]
/// will unwind.
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
pub(crate) struct RemoteHandle<T> {
    rx: Receiver<thread::Result<T>>,
    keep_running: Arc<AtomicBool>,
}

impl<T> RemoteHandle<T> {
    /// Drops this handle *without* canceling the underlying future.
    ///
    /// This method can be used if you want to drop the handle, but let the
    /// execution continue.
    pub(crate) fn forget(self) {
        self.keep_running.store(true, Ordering::SeqCst);
    }
}

impl<T: 'static> Future for RemoteHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        match ready!(self.rx.poll_unpin(cx)) {
            Ok(Ok(output)) => Poll::Ready(output),
            // the remote future panicked.
            Ok(Err(e)) => panic::resume_unwind(e),
            // The oneshot sender was dropped.
            Err(e) => panic::resume_unwind(Box::new(e)),
        }
    }
}

type SendMsg<Fut> = Result<<Fut as Future>::Output, Box<(dyn Any + Send + 'static)>>;

/// A future which sends its output to the corresponding `RemoteHandle`.
/// Created by [`remote_handle`](crate::future::FutureExt::remote_handle).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub(crate) struct Remote<Fut: Future> {
    tx: Option<Sender<SendMsg<Fut>>>,
    keep_running: Arc<AtomicBool>,
    future: CatchUnwind<AssertUnwindSafe<Fut>>,
}

impl<Fut: Future + fmt::Debug> fmt::Debug for Remote<Fut> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Remote")
            .field(&self.future)
            .finish()
    }
}

impl<Fut: Future + Unpin> Unpin for Remote<Fut> {}

impl<Fut: Future> Remote<Fut> {
    unsafe_pinned!(future: CatchUnwind<AssertUnwindSafe<Fut>>);
    unsafe_unpinned!(tx: Option<Sender<SendMsg<Fut>>>);
}

impl<Fut: Future> Future for Remote<Fut> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(_) = self.as_mut().tx().as_mut().unwrap().poll_canceled(cx) {
            if !self.keep_running.load(Ordering::SeqCst) {
                // Cancelled, bail out
                return Poll::Ready(())
            }
        }

        let output = ready!(self.as_mut().future().poll(cx));

        // if the receiving end has gone away then that's ok, we just ignore the
        // send error here.
        drop(self.as_mut().tx().take().unwrap().send(output));
        Poll::Ready(())
    }
}

#[ allow(dead_code) ]
pub(crate) fn remote_handle<Fut: Future>(future: Fut) -> (Remote<Fut>, RemoteHandle<Fut::Output>) {
    let (tx, rx) = oneshot::channel();
    let keep_running = Arc::new(AtomicBool::new(false));

    // Unwind Safety: See the docs for RemoteHandle.
    let wrapped = Remote {
        future: AssertUnwindSafe(future).catch_unwind(),
        tx: Some(tx),
        keep_running: keep_running.clone(),
    };

    (wrapped, RemoteHandle { rx, keep_running })
}