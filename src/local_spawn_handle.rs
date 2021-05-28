#[allow(unused_imports)]
//
use {
    crate::JoinHandle,
    futures_task::{LocalFutureObj, SpawnError},
    futures_util::{
        future::{abortable, FutureExt},
        task::LocalSpawnExt,
    },
    std::{
        future::Future,
        pin::Pin,
        rc::Rc,
        sync::{atomic::AtomicBool, Arc},
    },
};

/// This is similar to [`SpawnHandle`](crate::SpawnHandle) except that it allows spawning `!Send` futures. Please see
/// the docs on [`SpawnHandle`](crate::SpawnHandle).
pub trait LocalSpawnHandle<Out: 'static> {
    /// Spawn a future and return a [`JoinHandle`] that can be awaited for the output of the future.
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError>;
}

/// Let you spawn a !Send future and get a [JoinHandle] to await the output of a future.
pub trait LocalSpawnHandleExt<Out: 'static>: LocalSpawnHandle<Out> {
    /// Convenience trait for passing in a generic future to [`LocalSpawnHandle`]. Much akin to `LocalSpawn` and `LocalSpawnExt` in the
    /// futures library.
    fn spawn_handle_local(
        &self,
        future: impl Future<Output = Out> + 'static,
    ) -> Result<JoinHandle<Out>, SpawnError>;
}

impl<T, Out> LocalSpawnHandleExt<Out> for T
where
    T: LocalSpawnHandle<Out> + ?Sized,
    Out: 'static,
{
    fn spawn_handle_local(
        &self,
        future: impl Future<Output = Out> + 'static,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        self.spawn_handle_local_obj(LocalFutureObj::new(future.boxed_local()))
    }
}

impl<T: ?Sized, Out> LocalSpawnHandle<Out> for Box<T>
where
    T: LocalSpawnHandle<Out>,
    Out: 'static,
{
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        (**self).spawn_handle_local_obj(future)
    }
}

impl<T: ?Sized, Out> LocalSpawnHandle<Out> for Arc<T>
where
    T: LocalSpawnHandle<Out>,
    Out: 'static,
{
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        (**self).spawn_handle_local_obj(future)
    }
}

impl<T: ?Sized, Out> LocalSpawnHandle<Out> for Rc<T>
where
    T: LocalSpawnHandle<Out>,
    Out: 'static,
{
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        (**self).spawn_handle_local_obj(future)
    }
}

impl<T, Out> LocalSpawnHandle<Out> for &T
where
    T: LocalSpawnHandle<Out>,
    Out: 'static,
{
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        (**self).spawn_handle_local_obj(future)
    }
}

impl<T, Out> LocalSpawnHandle<Out> for &mut T
where
    T: LocalSpawnHandle<Out>,
    Out: 'static,
{
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        (**self).spawn_handle_local_obj(future)
    }
}

#[cfg(feature = "localpool")]
//
impl<Out: 'static> LocalSpawnHandle<Out> for crate::LocalSpawner {
    fn spawn_handle_local_obj(
        &self,
        future: LocalFutureObj<'static, Out>,
    ) -> Result<JoinHandle<Out>, SpawnError> {
        let (fut, handle) = future.remote_handle();

        self.spawn_local(fut)?;

        Ok(JoinHandle {
            inner: crate::join_handle::InnerJh::RemoteHandle(Some(handle)),
        })
    }
}

/// Let you spawn and get a [JoinHandle] to await the output of a future.
pub trait LocalSpawnHandleStatic {
    /// spawn and get a [JoinHandle] to await the output of a future.
    fn spawn_handle_local<Fut>(future: Fut) -> Result<JoinHandle<Fut::Output>, SpawnError>
    where
        Fut: Future + 'static,
        Fut::Output: 'static;
}
