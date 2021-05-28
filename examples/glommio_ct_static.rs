use async_executors::{GlommioCt, GlommioCtBuilder, SpawnStatic};
use futures::channel::{oneshot, oneshot::Sender};

fn lib_function<Exec: SpawnStatic>(tx: Sender<&'static str>) {
    Exec::spawn(async {
        tx.send("I can spawn from a library").expect("send string");
    })
    .expect("spawn task");
}

fn main() {
    // You provide the builder, and async_executors will set the right scheduler.
    // Of course you can set other configuration on the builder before.
    //
    let exec = GlommioCtBuilder::new();

    let program = async {
        let (tx, rx) = oneshot::channel();

        lib_function::<GlommioCt>(tx);
        println!("{}", rx.await.expect("receive on channel"));
    };

    exec.block_on(program);
}
