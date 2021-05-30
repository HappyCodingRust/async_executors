use std::fmt::Debug;

pub trait StaticRuntime: Debug + Send + Sync + Copy + Clone + Unpin + 'static {}
impl<T: Debug + Send + Sync + Copy + Clone + Unpin + 'static> StaticRuntime for T {}

pub trait WithRuntime {
    type Runtime: StaticRuntime;
}
