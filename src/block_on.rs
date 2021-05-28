use std::future::Future;

/// The entry point of the executor
pub trait BlockOn {
    /// The entry point of the executor
    fn block_on<F: Future>(&self, future: F) -> F::Output;
}
