use std::fmt::Debug;

pub trait StaticRuntime: Debug + Send + Sync + Copy + Clone {}
impl<T: Debug + Send + Sync + Copy + Clone> StaticRuntime for T {}
