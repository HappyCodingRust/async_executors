use futures_task::{FutureObj, LocalFutureObj};
use futures_util::future::RemoteHandle;
use futures_util::FutureExt;
use std::future::Future;
use std::sync::Arc;
use std::{boxed::Box, rc::Rc};

/// The `Spawn` trait allows for pushing futures onto an executor that will
/// run them to completion.
pub trait Spawn {
    /// Spawns a future that will be run to completion.
    ///
    /// # Errors
    ///
    /// The executor may be unable to spawn tasks. Spawn errors should
    /// represent relatively rare scenarios, such as the executor
    /// having been shut down so that it is no longer able to accept
    /// tasks.
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError>;

    /// Determines whether the executor is able to spawn new tasks.
    ///
    /// This method will return `Ok` when the executor is *likely*
    /// (but not guaranteed) to accept a subsequent spawn attempt.
    /// Likewise, an `Err` return means that `spawn` is likely, but
    /// not guaranteed, to yield an error.
    #[inline]
    fn status(&self) -> Result<(), SpawnError> {
        Ok(())
    }
}

/// The `LocalSpawn` is similar to [`Spawn`], but allows spawning futures
/// that don't implement `Send`.
pub trait LocalSpawn {
    /// Spawns a future that will be run to completion.
    ///
    /// # Errors
    ///
    /// The executor may be unable to spawn tasks. Spawn errors should
    /// represent relatively rare scenarios, such as the executor
    /// having been shut down so that it is no longer able to accept
    /// tasks.
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError>;

    /// Determines whether the executor is able to spawn new tasks.
    ///
    /// This method will return `Ok` when the executor is *likely*
    /// (but not guaranteed) to accept a subsequent spawn attempt.
    /// Likewise, an `Err` return means that `spawn` is likely, but
    /// not guaranteed, to yield an error.
    #[inline]
    fn status_local(&self) -> Result<(), SpawnError> {
        Ok(())
    }
}

/// An error that occurred during spawning.
#[derive(Copy, Clone)]
pub struct SpawnError {
    _priv: (),
}
impl SpawnError {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}
impl From<futures_util::task::SpawnError> for SpawnError {
    fn from(_: futures_util::task::SpawnError) -> Self {
        Self::new()
    }
}
impl std::fmt::Debug for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SpawnError").field(&"shutdown").finish()
    }
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Executor is shutdown")
    }
}

impl std::error::Error for SpawnError {}

impl SpawnError {
    /// Spawning failed because the executor has been shut down.
    pub fn shutdown() -> Self {
        Self { _priv: () }
    }

    /// Check whether spawning failed to the executor being shut down.
    pub fn is_shutdown(&self) -> bool {
        true
    }
}

impl<Sp: ?Sized + Spawn> Spawn for &Sp {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        Sp::spawn_obj(self, future)
    }

    fn status(&self) -> Result<(), SpawnError> {
        Sp::status(self)
    }
}

impl<Sp: ?Sized + Spawn> Spawn for &mut Sp {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        Sp::spawn_obj(self, future)
    }

    fn status(&self) -> Result<(), SpawnError> {
        Sp::status(self)
    }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for &Sp {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        Sp::spawn_local_obj(self, future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        Sp::status_local(self)
    }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for &mut Sp {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        Sp::spawn_local_obj(self, future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        Sp::status_local(self)
    }
}

impl<Sp: ?Sized + Spawn> Spawn for Box<Sp> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        (**self).spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
        (**self).status()
    }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for Box<Sp> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        (**self).spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        (**self).status_local()
    }
}

impl<Sp: ?Sized + Spawn> Spawn for Rc<Sp> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        (**self).spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
        (**self).status()
    }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for Rc<Sp> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        (**self).spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        (**self).status_local()
    }
}

impl<Sp: ?Sized + Spawn> Spawn for Arc<Sp> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        (**self).spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
        (**self).status()
    }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for Arc<Sp> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
        (**self).spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
        (**self).status_local()
    }
}

impl<Sp: ?Sized> SpawnExt for Sp where Sp: Spawn {}
impl<Sp: ?Sized> LocalSpawnExt for Sp where Sp: LocalSpawn {}

/// Extension trait for `Spawn`.
pub trait SpawnExt: Spawn {
    /// Spawns a task that polls the given future with output `()` to
    /// completion.
    ///
    /// This method returns a [`Result`] that contains a [`SpawnError`] if
    /// spawning fails.
    ///
    /// You can use [`spawn_with_handle`](SpawnExt::spawn_with_handle) if
    /// you want to spawn a future with output other than `()` or if you want
    /// to be able to await its completion.
    ///
    /// Note this method will eventually be replaced with the upcoming
    /// `Spawn::spawn` method which will take a `dyn Future` as input.
    /// Technical limitations prevent `Spawn::spawn` from being implemented
    /// today. Feel free to use this method in the meantime.
    ///
    /// ```
    /// use futures::executor::ThreadPool;
    /// use futures::task::SpawnExt;
    ///
    /// let executor = ThreadPool::new().unwrap();
    ///
    /// let future = async { /* ... */ };
    /// executor.spawn(future).unwrap();
    /// ```
    fn spawn<Fut>(&self, future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.spawn_obj(FutureObj::new(Box::new(future)))
    }

    /// Spawns a task that polls the given future to completion and returns a
    /// future that resolves to the spawned future's output.
    ///
    /// This method returns a [`Result`] that contains a [`RemoteHandle`](crate::future::RemoteHandle), or, if
    /// spawning fails, a [`SpawnError`]. [`RemoteHandle`](crate::future::RemoteHandle) is a future that
    /// resolves to the output of the spawned future.
    ///
    /// ```
    /// use futures::executor::{block_on, ThreadPool};
    /// use futures::future;
    /// use futures::task::SpawnExt;
    ///
    /// let executor = ThreadPool::new().unwrap();
    ///
    /// let future = future::ready(1);
    /// let join_handle_fut = executor.spawn_with_handle(future).unwrap();
    /// assert_eq!(block_on(join_handle_fut), 1);
    /// ```
    fn spawn_with_handle<Fut>(&self, future: Fut) -> Result<RemoteHandle<Fut::Output>, SpawnError>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send,
    {
        let (future, handle) = future.remote_handle();
        self.spawn(future)?;
        Ok(handle)
    }
}

/// Extension trait for `LocalSpawn`.
pub trait LocalSpawnExt: LocalSpawn {
    /// Spawns a task that polls the given future with output `()` to
    /// completion.
    ///
    /// This method returns a [`Result`] that contains a [`SpawnError`] if
    /// spawning fails.
    ///
    /// You can use [`spawn_with_handle`](SpawnExt::spawn_with_handle) if
    /// you want to spawn a future with output other than `()` or if you want
    /// to be able to await its completion.
    ///
    /// Note this method will eventually be replaced with the upcoming
    /// `Spawn::spawn` method which will take a `dyn Future` as input.
    /// Technical limitations prevent `Spawn::spawn` from being implemented
    /// today. Feel free to use this method in the meantime.
    ///
    /// ```
    /// use futures::executor::LocalPool;
    /// use futures::task::LocalSpawnExt;
    ///
    /// let executor = LocalPool::new();
    /// let spawner = executor.spawner();
    ///
    /// let future = async { /* ... */ };
    /// spawner.spawn_local(future).unwrap();
    /// ```
    fn spawn_local<Fut>(&self, future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = ()> + 'static,
    {
        self.spawn_local_obj(LocalFutureObj::new(Box::new(future)))
    }

    /// Spawns a task that polls the given future to completion and returns a
    /// future that resolves to the spawned future's output.
    ///
    /// This method returns a [`Result`] that contains a [`RemoteHandle`](crate::future::RemoteHandle), or, if
    /// spawning fails, a [`SpawnError`]. [`RemoteHandle`](crate::future::RemoteHandle) is a future that
    /// resolves to the output of the spawned future.
    ///
    /// ```
    /// use futures::executor::LocalPool;
    /// use futures::task::LocalSpawnExt;
    ///
    /// let mut executor = LocalPool::new();
    /// let spawner = executor.spawner();
    ///
    /// let future = async { 1 };
    /// let join_handle_fut = spawner.spawn_local_with_handle(future).unwrap();
    /// assert_eq!(executor.run_until(join_handle_fut), 1);
    /// ```
    fn spawn_local_with_handle<Fut>(
        &self,
        future: Fut,
    ) -> Result<RemoteHandle<Fut::Output>, SpawnError>
    where
        Fut: Future + 'static,
    {
        let (future, handle) = future.remote_handle();
        self.spawn_local(future)?;
        Ok(handle)
    }
}

/// The `SpawnStatic` trait allows for pushing futures onto an executor that will
/// run them to completion. Except that this is used for ZST as type
pub trait SpawnStatic {
    /// Spawns a future that will be run to completion
    fn spawn<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: Send + 'static;
}

/// The `LocalSpawnStatic` is similar to [`SpawnStatic`], but allows spawning futures
/// that don't implement `Send`.
pub trait LocalSpawnStatic {
    /// Spawns a future that will be run to completion
    fn spawn_local<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static;
}
