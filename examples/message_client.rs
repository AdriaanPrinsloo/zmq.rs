mod async_helpers;

use std::convert::TryInto;
use std::error::Error;
use zeromq::Socket;
use zeromq::{BlockingRecv, BlockingSend};

#[async_helpers::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut socket = zeromq::ReqSocket::new();
    socket
        .connect("tcp://127.0.0.1:5559")
        .await
        .expect("Failed to connect");

    socket.send("Hello".into()).await?;
    let repl: String = socket.recv().await?.try_into()?;
    dbg!(repl);

    socket.send("NewHello".into()).await?;
    let repl: String = socket.recv().await?.try_into()?;
    dbg!(repl);
    Ok(())
}
