use crate::{client::Client, server::Server, transport::tokio_channels::TokioChannelTransport};
use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[ctor::ctor]
fn init_test_logging() {
    env_logger::init();
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) enum Request {
    Ping(u32),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) enum Response {
    Ping(u32),
}

#[tokio::test]
async fn basic_rpc_transaction() {
    let (t1, t2) = TokioChannelTransport::new_pair(256);

    let mut client = Client::<_, Request, Response>::new(t1);
    let mut server = Server::<_, Request, Response>::new(t2);

    let done = Arc::new(AtomicBool::new(false));

    let run_server = {
        let done = done.clone();
        async move {
            loop {
                if done.load(Ordering::Relaxed) {
                    break;
                }

                let request = server
                    .wait_for_request(Duration::from_secs(5))
                    .await
                    .unwrap();
                server
                    .send_response(match request {
                        Request::Ping(i) => Response::Ping(i),
                    })
                    .await
                    .unwrap();
            }
        }
    };

    let run_client = async move {
        let response = client
            .call(Request::Ping(42), Duration::from_millis(500))
            .await
            .unwrap();
        assert_eq!(response, Response::Ping(42));
        done.store(true, Ordering::Relaxed)
    };

    tokio::join!(run_server, run_client);
}

#[tokio::test]
async fn recover_from_out_of_sync() {
    let (t1, mut t2) = TokioChannelTransport::new_pair(256);

    // Send some garbage data, which could be half a message that was interrupted
    t2.transmit_raw(b"lol wtf?").await.unwrap();

    let mut client = Client::<_, Request, Response>::new(t1);
    let mut server = Server::<_, Request, Response>::new(t2);

    let done = Arc::new(AtomicBool::new(false));

    let run_server = {
        let done = done.clone();
        async move {
            loop {
                if done.load(Ordering::Relaxed) {
                    break;
                }

                let request = server
                    .wait_for_request(Duration::from_secs(5))
                    .await
                    .unwrap();
                server
                    .send_response(match request {
                        Request::Ping(i) => Response::Ping(i),
                    })
                    .await
                    .unwrap();
            }
        }
    };

    let run_client = async move {
        let response = client
            .call(Request::Ping(42), Duration::from_millis(500))
            .await
            .unwrap();
        assert_eq!(response, Response::Ping(42));
        done.store(true, Ordering::Relaxed)
    };

    tokio::join!(run_server, run_client);
}
