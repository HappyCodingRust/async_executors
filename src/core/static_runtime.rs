use std::fmt::Debug;

pub trait StaticRuntime: Debug + Send + Sync + Copy + Clone + Unpin + Default + 'static {}
impl<T: Debug + Send + Sync + Copy + Clone + Unpin + Default + 'static> StaticRuntime for T {}

pub trait WithRuntime {
    type Runtime: StaticRuntime;
}
