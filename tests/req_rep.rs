use zeromq::prelude::*;
use zeromq::RepSocket;

use futures::StreamExt;
use std::convert::TryInto;
use std::error::Error;
use std::time::Duration;

async fn run_rep_server(mut rep_socket: RepSocket) -> Result<(), Box<dyn Error>> {
    println!("Started rep server on tcp://127.0.0.1:5557");

    for i in 0..10i32 {
        let mess: String = rep_socket.recv().await?.try_into()?;
        rep_socket
            .send(format!("{} Rep - {}", mess, i).into())
            .await?;
    }
    // yield for a moment to ensure that server has some time to flush socket
    let errs = rep_socket.close().await;
    if !errs.is_empty() {
        panic!("Could not unbind socket: {:?}", errs);
    }
    Ok(())
}

#[tokio::test]
async fn test_req_rep_sockets() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::try_init().ok();

    let mut rep_socket = zeromq::RepSocket::new();
    let monitor = rep_socket.monitor();
    let endpoint = rep_socket.bind("tcp://localhost:0").await?;
    println!("Started rep server on {}", endpoint);

    tokio::spawn(async {
        run_rep_server(rep_socket).await.unwrap();
    });

    let mut req_socket = zeromq::ReqSocket::new();
    req_socket.connect(endpoint.to_string().as_str()).await?;

    for i in 0..10i32 {
        req_socket.send(format!("Req - {}", i).into()).await?;
        let repl: String = req_socket.recv().await?.try_into()?;
        assert_eq!(format!("Req - {} Rep - {}", i, i), repl)
    }
    req_socket.close().await;
    let events: Vec<_> = monitor.collect().await;
    assert_eq!(2, events.len(), "{:?}", &events);
    Ok(())
}

#[tokio::test]
async fn test_many_req_rep_sockets() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::try_init().ok();

    let mut rep_socket = zeromq::RepSocket::new();
    let endpoint = rep_socket.bind("tcp://localhost:0").await?;
    println!("Started rep server on {}", endpoint);

    for i in 0..100i32 {
        let cloned_endpoint = endpoint.to_string();
        tokio::spawn(async move {
            // yield for a moment to ensure that server has some time to open socket
            tokio::time::sleep(Duration::from_millis(100)).await;
            let mut req_socket = zeromq::ReqSocket::new();
            req_socket.connect(&cloned_endpoint).await.unwrap();

            for j in 0..100i32 {
                req_socket
                    .send(format!("Socket {} Req - {}", i, j).into())
                    .await
                    .unwrap();
                let repl: String = req_socket.recv().await.unwrap().try_into().unwrap();
                assert_eq!(format!("Socket {} Req - {} Rep", i, j), repl)
            }
            drop(req_socket);
        });
    }

    for _ in 0..10000i32 {
        let mess: String = rep_socket.recv().await?.try_into()?;
        rep_socket.send(format!("{} Rep", mess).into()).await?;
    }
    Ok(())
}
