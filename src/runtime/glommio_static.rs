use crate::{
    JoinHandle, LocalSpawnHandleStatic, LocalSpawnStatic, SpawnBlockingStatic, SpawnError,
    SpawnHandleStatic, SpawnStatic, YieldNowStatic,
};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use glommio_crate::Task;
use std::future::Future;

/// A simple glommio runtime builder
#[derive(Debug, Clone, Copy)]
pub struct Glommio;

impl LocalSpawnStatic for Glommio {
    fn spawn_local<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
    {
        glommio_crate::Task::local(future).detach();
        Ok(())
    }
}

impl LocalSpawnHandleStatic for Glommio {
    fn spawn_handle_local<Output, Fut>(future: Fut) -> Result<JoinHandle<Output>, SpawnError>
    where
        Fut: Future<Output = Output> + 'static,
        Output: 'static,
    {
        let (remote, handle) = future.remote_handle();
        Task::local(remote).detach();
        Ok(handle.into())
    }
}

impl SpawnStatic for Glommio {
    fn spawn<Output, Fut>(future: Fut) -> Result<(), SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: Send + 'static,
    {
        glommio_crate::Task::local(future).detach();
        Ok(())
    }
}

impl SpawnHandleStatic for Glommio {
    fn spawn_handle<Output, Fut>(future: Fut) -> Result<JoinHandle<Output>, SpawnError>
    where
        Fut: Future<Output = Output> + Send + 'static,
        Output: 'static + Send,
    {
        let (remote, handle) = future.remote_handle();
        glommio_crate::Task::local(remote).detach();
        Ok(handle.into())
    }
}
impl YieldNowStatic for Glommio {
    fn yield_now() -> BoxFuture<'static, ()> {
        Box::pin(Task::<()>::yield_if_needed())
    }
}

impl SpawnBlockingStatic for Glommio {
    fn spawn_blocking<T: Send + 'static>(
        func: impl FnOnce() -> T + Send + 'static,
    ) -> Result<JoinHandle<T>, SpawnError> {
        let (remote, handle) = async { func() }.remote_handle();
        std::thread::spawn(move || {
            bind_to_cpu_set(None).expect("Unbind core affinity error");
            futures_executor::block_on(remote)
        });
        Ok(handle.into())
    }
}

macro_rules! to_io_error {
    ($error:expr) => {{
        match $error {
            Ok(x) => Ok(x),
            Err(nix::Error::Sys(_)) => Err(std::io::Error::last_os_error()),
            Err(nix::Error::InvalidUtf8) => {
                Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
            }
            Err(nix::Error::InvalidPath) => {
                Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
            }
            Err(nix::Error::UnsupportedOperation) => {
                Err(std::io::Error::from(std::io::ErrorKind::Other))
            }
        }
    }};
}
fn bind_to_cpu_set(cpus: impl IntoIterator<Item = usize>) -> std::io::Result<()> {
    let mut cpuset = nix::sched::CpuSet::new();
    for cpu in cpus {
        to_io_error!(&cpuset.set(cpu))?;
    }
    let pid = nix::unistd::Pid::from_raw(0);
    to_io_error!(nix::sched::sched_setaffinity(pid, &cpuset)).map_err(Into::into)
}
