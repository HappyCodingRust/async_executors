use
{
	futures::task     :: { Spawn, SpawnExt          } ,
	futures::executor :: { block_on, ThreadPool     } ,
	futures::channel  :: { oneshot, oneshot::Sender } ,
};


fn lib_function( exec: impl Spawn, tx: Sender<&'static str> )
{
	exec.spawn( async
	{
		tx.send( "I can spawn from a library" ).expect( "send string" );

	}).expect( "spawn task" );
}


fn main()
{
	// You provide the builder, and async_executors will set the right scheduler.
	// Of course you can set other configuration on the builder before.
	//
	let exec = ThreadPool::new().expect( "create futures threadpool" );

	let program = async
	{
		let (tx, rx) = oneshot::channel();

		lib_function( &exec, tx );
		println!( "{}", rx.await.expect( "receive on channel" ) );
	};

	block_on( program );
}