use futures_util::future::BoxFuture;

/// Indicates that a runtime can yield
pub trait YieldNow {
    /// yield now
    fn yield_now<'a>(&'a self) -> BoxFuture<'a, ()>;
}
