use crate::{
    JoinHandle, LocalSpawnHandleStatic, LocalSpawnStatic, SpawnBlockingStatic, SpawnError,
    SpawnHandleStatic, SpawnStatic, YieldNowStatic,
};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use glommio_crate::Task;
use nix::sched::CpuSet;
use std::cell::Cell;
use std::future::Future;
use std::marker::PhantomData;
use std::rc::Rc;

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
        let cpu_set = DEFAULT_CPU_SET.with(|x| x.clone().into_inner()).unwrap();
        std::thread::spawn(move || {
            bind_to_cpu_set(cpu_set).expect("Unbind core affinity error");
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
fn bind_to_cpu_set(cpuset: CpuSet) -> std::io::Result<()> {
    let pid = nix::unistd::Pid::this();
    to_io_error!(nix::sched::sched_setaffinity(pid, &cpuset))
}

thread_local! {
    static DEFAULT_CPU_SET: Cell<Option<CpuSet>> = Cell::new(None);
}
fn set_default_cpu() -> std::io::Result<()> {
    let pid = nix::unistd::Pid::this();
    let set = to_io_error!(nix::sched::sched_getaffinity(pid))?;

    DEFAULT_CPU_SET.with(|x| match x.clone().into_inner() {
        Some(_) => {
            panic!("Default cpu set of this thread has already been set")
        }
        None => x.set(Some(set)),
    });
    Ok(())
}

fn clean_default_cpu() -> std::io::Result<()> {
    DEFAULT_CPU_SET.with(|x| x.set(None));
    Ok(())
}
#[derive(Debug)]
pub(crate) struct CoreAffinityGuard {
    phantom: PhantomData<Rc<()>>,
}

impl CoreAffinityGuard {
    pub fn new() -> std::io::Result<Self> {
        set_default_cpu()?;
        Ok(Self {
            phantom: Default::default(),
        })
    }
}
impl Drop for CoreAffinityGuard {
    fn drop(&mut self) {
        clean_default_cpu().unwrap();
    }
}
