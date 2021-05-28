#[ allow(unused_imports) ] // some imports are conditional on features
//
use
{
	std         :: { future::Future, sync::atomic::{ AtomicBool, Ordering } } ,
	std         :: { task::{ Poll, Context }, pin::Pin                      } ,
	futures_util:: { future::{ AbortHandle, Aborted, RemoteHandle }, ready  } ,
};


#[ cfg( feature = "async_global" ) ]
//
use async_global_executor::{ Task as AsyncGlobalTask };

#[ cfg( feature = "async_std" ) ]
//
use async_std_crate::{ task::JoinHandle as AsyncStdJoinHandle };

#[ cfg(any( feature = "tokio_tp", feature = "tokio_ct" )) ]
//
use tokio::{ task::JoinHandle as TokioJoinHandle };

#[cfg(feature = "glommio")]
//
use glommio_crate::Task as GlommioTask;

/// A Join Handle
pub trait AsyncJoinHandle: Future {

	/// Drops this handle without canceling the underlying future.
	///
	/// This method can be used if you want to drop the handle, but let the execution continue.
	fn detach(self);
}

/// A framework agnostic JoinHandle type. Cancels the future on dropping the handle.
/// You can call [`detach`](JoinHandle::detach) to leave the future running when dropping the handle.
///
/// This leverages the performance gains from the native join handles compared to
/// [RemoteHandle](futures_util::future::RemoteHandle) where possible.
///
/// It does wrap futures in [Abortable](futures_util::future::Abortable) where needed as
/// [tokio] and [async-std](async_std_crate) don't support canceling out of the box.
///
/// # Panics
///
/// There is an inconsistency between executors when it comes to a panicking task.
/// Generally we unwind the thread on which the handle is awaited when a task panics,
/// but async-std will also let the executor thread unwind. No `catch_unwind` was added to
/// bring async-std in line with the other executors here.
///
/// Awaiting the JoinHandle can also panic if you drop the executor before it completes.
//
#[ derive( Debug ) ]
//
#[ must_use = "JoinHandle will cancel your future when dropped." ]
//
pub struct JoinHandle<T> { inner: InnerJh<T> }

impl<T> JoinHandle<T> {
	pub(crate) fn new(inner: InnerJh<T>) -> Self {
		Self {
			inner
		}
	}
}

impl<T: 'static> AsyncJoinHandle for JoinHandle<T> {
	fn detach(self) {
		Self::detach(self)
	}
}


#[ derive(Debug) ] #[ allow(dead_code) ]
//
pub(crate) enum InnerJh<T>
{
	/// Wrapper around tokio JoinHandle.
	//
	#[ cfg(any( feature = "tokio_tp", feature = "tokio_ct" )) ]
	//
	Tokio
	{
		handle  : TokioJoinHandle<T> ,
		detached: AtomicBool         ,
	},

	/// Wrapper around AsyncStd JoinHandle.
	//
	#[ cfg( feature = "async_global" ) ]
	//
	AsyncGlobal
	{
		task: Option< AsyncGlobalTask<T> > ,
	},

	/// Wrapper around AsyncStd JoinHandle.
	//
	#[ cfg( feature = "async_std" ) ]
	//
	AsyncStd
	{
		handle  : AsyncStdJoinHandle<Result<T, Aborted>> ,
		a_handle: AbortHandle                            ,
		detached: AtomicBool                             ,
	},
	/// Wrapper around AsyncStd JoinHandle.
	//
	#[ cfg( feature = "glommio" ) ]
	//
	Glommio
	{
		task: Option<GlommioTask<T>>,
	},

	/// Wrapper around futures RemoteHandle.
	//
	RemoteHandle( Option<RemoteHandle<T>> ),
}

impl<T> JoinHandle<T>
{
	/// Drops this handle without canceling the underlying future.
	///
	/// This method can be used if you want to drop the handle, but let the execution continue.
	//
	pub fn detach( mut self )
	{
		match &mut self.inner
		{
			#[ cfg(any( feature = "tokio_tp", feature = "tokio_ct" )) ]
			//
			InnerJh::Tokio{ ref detached, .. } =>
			{
				// only other use of this is in Drop impl and we consume self here,
				// so there cannot be any race as this does not sync things across threads,
				// hence Relaxed ordering.
				//
				detached.store( true, Ordering::Relaxed );
			}

			#[ cfg( feature = "async_global" ) ] InnerJh::AsyncGlobal{ task } =>
			{
				let task = task.take();
				task.unwrap().detach();
			}

			#[ cfg( feature = "async_std" ) ] InnerJh::AsyncStd{ ref detached, .. } =>
			{
				detached.store( true, Ordering::Relaxed );
			}

			#[ cfg( feature = "glommio" ) ] InnerJh::Glommio { task } => {
				task.take().map(|x| x.detach());
			}

			InnerJh::RemoteHandle( handle ) =>
			{
				if let Some(rh) = handle.take() { rh.forget() };
			}

		}
	}
}



impl<T: 'static> Future for JoinHandle<T>
{
	type Output = T;

	fn poll( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Self::Output>
	{
		match &mut self.get_mut().inner
		{
			#[ cfg(any( feature = "tokio_tp", feature = "tokio_ct" )) ]
			//
			InnerJh::Tokio{ handle, .. } =>
			{
				match ready!( Pin::new( handle ).poll( cx ) )
				{
					Ok (t) => Poll::Ready( t ),

					Err(e) =>
					{
						panic!( "Task has been canceled. Are you dropping the executor to early? Error: {}", e );
					}
				}
			}


			#[ cfg( feature = "async_std" ) ] InnerJh::AsyncStd{ handle, .. } =>
			{
				match ready!( Pin::new( handle ).poll( cx ) )
				{
					Ok (t) => Poll::Ready( t ),
					Err(_) => unreachable!(),
				}
			}


			#[ cfg( feature = "async_global" ) ] InnerJh::AsyncGlobal{ task, .. } =>
			{
				Pin::new( task.as_mut().unwrap() ).poll( cx )
			}


			#[ cfg( feature = "glommio" ) ] InnerJh::Glommio { task } => {
				if let Some(task) = task {
					Pin::new(task).poll(cx)
				} else {
					unreachable!("Why would it ever happen?")
				}
			}
			InnerJh::RemoteHandle( ref mut handle ) => Pin::new( handle ).as_pin_mut().expect( "no polling after detach" ).poll( cx ),
		}
	}
}



impl<T> Drop for JoinHandle<T>
{
	// see reasoning about Relaxed atomic in detach().
	//
	fn drop( &mut self )
	{
		match &mut self.inner
		{
			#[ cfg(any( feature = "tokio_tp", feature = "tokio_ct" )) ]
			//
			InnerJh::Tokio{ handle, detached, .. } =>

				if !detached.load( Ordering::Relaxed ) { handle.abort() },


			#[ cfg( feature = "async_std" ) ] InnerJh::AsyncStd { a_handle, detached, .. } =>

				if !detached.load( Ordering::Relaxed ) { a_handle.abort() },


			// Nothing needs to be done, just drop it.
			//
			#[ cfg( feature = "async_global" ) ] InnerJh::AsyncGlobal { .. } => {}

			#[ cfg( feature = "glommio" ) ] InnerJh::Glommio { .. } => {}

			InnerJh::RemoteHandle( _ ) => {},
		};
	}
}


#[cfg(test)]
//
mod tests {
	use super::*;

	// It's important that this is not Send, as we allow spawning !Send futures on it.
	//
	static_assertions::assert_impl_all!(JoinHandle<()>: Send);
}