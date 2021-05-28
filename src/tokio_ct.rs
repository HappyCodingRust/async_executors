use crate::block_on::BlockOn;
use crate::{LocalSpawnHandleStatic, LocalSpawnStatic, SpawnHandleStatic, SpawnStatic, YieldNow};
use futures_util::future::BoxFuture;
use {
    crate::{join_handle::InnerJh, JoinHandle, LocalSpawnHandle, SpawnHandle},
    futures_task::{FutureObj, LocalFutureObj, LocalSpawn, Spawn, SpawnError},
    std::{future::Future, sync::atomic::AtomicBool, sync::Arc},
    tokio::{runtime::Runtime, task::LocalSet},
};

/// An executor that uses a [`tokio::runtime::Runtime`] with the [current thread](tokio::runtime::Builder::new_current_thread)
/// and a [`tokio::task::LocalSet`]. Can spawn `!Send` futures.
///
/// ## Creation of the runtime
///
/// You must use [`TokioCtBuilder`](crate::TokioCtBuilder) to create the executor.
///
/// ```
/// // Make sure to set the `tokio_ct` feature on async_executors.
/// //
/// use
/// {
///    async_executors :: { TokioCt, TokioCtBuilder, LocalSpawnHandleExt, BlockOn } ,
///    tokio           :: { runtime::Builder                             } ,
///    std             :: { rc::Rc                                       } ,
/// };
///
/// // You must use the builder. This guarantees that TokioCt is always backed by a single threaded runtime.
/// // You can set other configurations by calling `tokio_builder()` on TokioCtBuilder, so you get
/// // access to the `tokio::runtime::Builder`.
/// //
/// let exec = TokioCtBuilder::new().build().expect( "create tokio runtime" );
///
/// // block_on takes a &self, so if you need to `async move`,
/// // just clone it for use inside the async block.
/// //
/// exec.block_on( async
/// {
///    let not_send = async { let rc = Rc::new(()); };
///
///    // We can spawn !Send futures here.
///    //
///    let join_handle = exec.spawn_handle_local( not_send ).expect( "spawn" );
///
///    join_handle.await;
/// });
///```
///
/// ## Unwind Safety.
///
/// When a future spawned on this wrapper panics, the panic will be caught by tokio in the poll function.
///
/// You must only spawn futures to this API that are unwind safe. Tokio will wrap spawned tasks in
/// [`std::panic::AssertUnwindSafe`] and wrap the poll invocation with [`std::panic::catch_unwind`].
///
/// They reason that this is fine because they require `Send + 'static` on the task. As far
/// as I can tell this is wrong. Unwind safety can be circumvented in several ways even with
/// `Send + 'static` (eg. `parking_lot::Mutex` is `Send + 'static` but `!UnwindSafe`).
///
/// You should make sure that if your future panics, no code that lives on after the panic,
/// nor any destructors called during the unwind can observe data in an inconsistent state.
///
/// Note: the future running from within `block_on` as opposed to `spawn` does not exhibit this behavior and will panic
/// the current thread.
///
/// Note that these are logic errors, not related to the class of problems that cannot happen
/// in safe rust (memory safety, undefined behavior, unsoundness, data races, ...). See the relevant
/// [catch_unwind RFC](https://github.com/rust-lang/rfcs/blob/master/text/1236-stabilize-catch-panic.md)
/// and it's discussion threads for more info as well as the documentation of [std::panic::UnwindSafe]
/// for more information.
///
//
#[derive(Debug, Clone)]
//
#[cfg_attr(nightly, doc(cfg(feature = "tokio_ct")))]
//
pub struct TokioCt {
    pub(crate) exec: Arc<Runtime>,
    pub(crate) local: Arc<LocalSet>,
}
impl TokioCt {
    /// This is the entry point for this executor. Once this call returns, no remaining tasks shall be polled anymore.
    /// However the tasks stay in the executor, so if you make a second call to `block_on` with a new task, the older
    /// tasks will start making progress again.
    ///
    /// For simplicity, it's advised to just create top level task that you run through `block_on` and make sure your
    /// program is done when it returns.
    ///
    /// See: [tokio::runtime::Runtime::block_on]
    ///
    /// ## Panics
    ///
    /// This function will panic if it is called from an async context, including but not limited to making a nested
    /// call. It will also panic if the provided future panics.
    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        self.exec.block_on(self.local.run_until(f))
    }
}
impl BlockOn for TokioCt {
    fn block_on<F: Future>(&self, f: F) -> F::Output {
        Self::block_on(self, f)
    }
}
impl Spawn for TokioCt {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        // We drop the JoinHandle, so the task becomes detached.
        //
        let _ = self.local.spawn_local(future);

        Ok(())
    }
}
impl SpawnStatic for TokioCt {
    fn spawn<Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let _ = tokio::task::spawn(future);
        Ok(())
    }
}
impl LocalSpawn for TokioCt {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        // We drop the JoinHandle, so the task becomes detached.
        //
        let _ = self.local.spawn_local(future);

        Ok(())
    }
}
impl LocalSpawnStatic for TokioCt {
    fn spawn_local<Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future + 'static,
        Fut::Output: 'static,
    {
        let _ = tokio::task::spawn_local(future);
        Ok(())
    }
}

impl<Out: 'static + Send> SpawnHandle<Out> for TokioCt {
    fn spawn_handle_obj(
        &self,
        future: FutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Ok(JoinHandle {
            inner: InnerJh::Tokio {
                handle: self.exec.spawn(future),
                detached: AtomicBool::new(false),
            },
        })
    }
}
impl SpawnHandleStatic for TokioCt {
    fn spawn_handle<Fut>(future: Fut) -> Result<JoinHandle<Fut::Output>, SpawnError>
    where
        Fut: Future + Send + 'static,
        Fut::Output: 'static + Send,
    {
        Ok(JoinHandle {
            inner: InnerJh::Tokio {
                handle: tokio::task::spawn(future),
                detached: AtomicBool::new(false),
            },
        })
    }
}
impl<Out: 'static> LocalSpawnHandle<Out> for TokioCt {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        Ok(JoinHandle {
            inner: InnerJh::Tokio {
                handle: self.local.spawn_local(future),
                detached: AtomicBool::new(false),
            },
        })
    }
}
impl LocalSpawnHandleStatic for TokioCt {
    fn spawn_handle_local<Fut>(future: Fut) -> Result<JoinHandle<Fut::Output>, SpawnError>
    where
        Fut: Future + 'static,
        Fut::Output: 'static,
    {
        Ok(JoinHandle {
            inner: InnerJh::Tokio {
                handle: tokio::task::spawn_local(future),
                detached: AtomicBool::new(false),
            },
        })
    }
}

impl YieldNow for TokioCt {
    fn yield_now<'a>(&'a self) -> BoxFuture<'a, ()> {
        Box::pin(tokio::task::yield_now())
    }
}

#[cfg(test)]
//
mod tests {
    use super::*;

    // It's important that this is not Send, as we allow spawning !Send futures on it.
    //
    static_assertions::assert_not_impl_any!(TokioCt: Send, Sync);
}
