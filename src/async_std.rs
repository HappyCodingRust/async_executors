use
{
	crate :: { import::* } ,
};


/// An executor that spawns tasks on async-std. In contrast to the other executors, this one
/// is not self contained, because async-std does not provide an API that allows that.
/// So the threadpool is global.
//
#[ derive( Clone, Default ) ]
//
pub struct AsyncStd {}



impl Spawn for AsyncStd
{
	fn spawn_obj( &self, future: FutureObj<'static, ()> ) -> Result<(), FutSpawnErr>
	{
		async_std_crate::task::spawn( future );

		Ok(())
	}
}


impl std::fmt::Debug for AsyncStd
{
	fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result
	{
		write!( f, "AsyncStd threadpool" )
	}
}
