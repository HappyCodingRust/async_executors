use crate::StaticRuntime;
use futures_util::future::BoxFuture;

/// Indicates that a runtime can yield
pub trait YieldNow {
    /// yield now
    fn yield_now<'a>(&'a self) -> BoxFuture<'a, ()>;
}
/// Indicates that a runtime can yield
pub trait YieldNowStatic: StaticRuntime {
    /// yield now
    fn yield_now() -> BoxFuture<'static, ()>;
}
impl<T: YieldNowStatic> YieldNow for T {
    fn yield_now<'a>(&'a self) -> BoxFuture<'a, ()> {
        T::yield_now()
    }
}
