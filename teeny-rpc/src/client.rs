use crate::{debug, trace, transport::Transport, warn, RpcMessage, RpcMessageKind};
use core::{marker::PhantomData, time::Duration};
use serde::{Deserialize, Serialize};

pub struct Client<
    'de,
    T,
    REQ: Serialize + Deserialize<'de> + PartialEq,
    RESP: Serialize + Deserialize<'de> + PartialEq,
> where
    T: Transport<RpcMessage<REQ, RESP>>,
{
    transport: T,
    seq: u32,
    _request: PhantomData<REQ>,
    _response: PhantomData<RESP>,
    _de_lifetime: PhantomData<&'de ()>,
}

impl<
        'de,
        T,
        REQ: Serialize + Deserialize<'de> + PartialEq + Clone,
        RESP: Serialize + Deserialize<'de> + PartialEq + Clone,
    > Client<'de, T, REQ, RESP>
where
    T: Transport<RpcMessage<REQ, RESP>>,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            seq: 0,
            _request: PhantomData,
            _response: PhantomData,
            _de_lifetime: PhantomData,
        }
    }

    pub async fn call(&mut self, request: REQ, timeout: Duration) -> Result<RESP, crate::Error> {
        let flushed_bytes = self.transport.flush(Duration::from_millis(5)).await?;
        debug!("Flushed {} bytes prior to request", flushed_bytes);

        self.seq = self.seq.wrapping_add(1);

        let request = RpcMessage {
            seq: self.seq,
            kind: RpcMessageKind::Request { payload: request },
        };
        debug!("New request {}", request.seq);

        // Send the request
        trace!("Sending request {}", request.seq,);
        self.transport.transmit_message(request.clone()).await?;

        // Receive the request acknowledgement
        match self.transport.receive_message(crate::ACK_TIMEOUT).await {
            Ok(ack) => {
                if ack.kind != RpcMessageKind::RequestAck {
                    return Err(crate::Error::IncorrectMessageType);
                }
                if ack.seq != self.seq {
                    return Err(crate::Error::IncorrectSequenceNumber {
                        expected: self.seq,
                        actual: ack.seq,
                    });
                }

                trace!("Received ack for request {}", request.seq);
            }
            Err(crate::Error::Timeout) => {
                warn!("Timeout waiting for ack for request {}", request.seq);
                return Err(crate::Error::NoAck);
            }
            Err(e) => {
                warn!("Error waiting for ack for request {}: {}", request.seq, e);
                return Err(crate::Error::NoAck);
            }
        }

        // Receive the response
        trace!("Waiting for response for request {}", request.seq);
        let resp = self.transport.receive_message(timeout).await?;
        if resp.seq != self.seq {
            return Err(crate::Error::IncorrectSequenceNumber {
                expected: self.seq,
                actual: resp.seq,
            });
        }

        if let RpcMessageKind::Response { payload } = resp.kind {
            // Send the response acknowledgement
            trace!("Sending ack for response {}", request.seq);
            let ack = RpcMessage {
                seq: self.seq,
                kind: RpcMessageKind::ResponseAck,
            };
            self.transport.transmit_message(ack).await?;

            debug!("Request {} complete", request.seq);
            Ok(payload)
        } else {
            Err(crate::Error::IncorrectMessageType)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        client::Client,
        test::{Request, Response},
        transport::{tokio_channels::TokioChannelTransport, Transport},
        RpcMessage, RpcMessageKind, ACK_TIMEOUT,
    };
    use core::time::Duration;

    #[tokio::test]
    async fn basic() {
        let (t1, mut t2) = TokioChannelTransport::new_pair(256);

        let mut client = Client::<_, Request, Response>::new(t1);

        let run_server = async move {
            let msg = t2.receive_message(Duration::from_millis(20)).await.unwrap();
            assert_eq!(msg.seq, 1);

            t2.transmit_message(RpcMessage {
                seq: 1,
                kind: RpcMessageKind::RequestAck,
            })
            .await
            .unwrap();

            t2.transmit_message(RpcMessage {
                seq: 1,
                kind: RpcMessageKind::Response {
                    payload: Response::Ping(42),
                },
            })
            .await
            .unwrap();

            let msg = t2.receive_message(Duration::ZERO).await.unwrap();
            assert_eq!(msg.seq, 1);
            assert_eq!(msg.kind, RpcMessageKind::ResponseAck);
        };

        let run_client = async move {
            let response = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await
                .unwrap();
            assert_eq!(response, Response::Ping(42));
        };

        tokio::join!(run_server, run_client);
    }

    #[tokio::test]
    async fn no_ack() {
        let (t1, mut t2) = TokioChannelTransport::new_pair(256);

        let mut client = Client::<_, Request, Response>::new(t1);

        let run_server = async move {
            let expected_request = RpcMessage {
                seq: 1,
                kind: RpcMessageKind::Request {
                    payload: Request::Ping(42),
                },
            };

            assert_eq!(
                t2.receive_message(Duration::from_millis(20)).await.unwrap(),
                expected_request
            );

            tokio::time::sleep(ACK_TIMEOUT).await;

            // Exit when client goes away
            assert_eq!(
                t2.receive_message(Duration::from_secs(30)).await,
                Err(crate::Error::TransportError)
            );
        };

        let run_client = async move {
            let result = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await;
            assert_eq!(result, Err(crate::Error::NoAck));
        };

        tokio::join!(run_server, run_client);
    }

    #[tokio::test]
    async fn no_response() {
        let (t1, mut t2) = TokioChannelTransport::new_pair(256);

        let mut client = Client::<_, Request, Response>::new(t1);

        let run_server = async move {
            assert_eq!(
                t2.receive_message(Duration::from_millis(10)).await.unwrap(),
                RpcMessage {
                    seq: 1,
                    kind: RpcMessageKind::Request {
                        payload: Request::Ping(42),
                    },
                }
            );

            t2.transmit_message(RpcMessage {
                seq: 1,
                kind: RpcMessageKind::RequestAck,
            })
            .await
            .unwrap();

            // Wait for the request to timeout
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Exit when client goes away
            assert_eq!(
                t2.receive_message(Duration::from_secs(30)).await,
                Err(crate::Error::TransportError)
            );
        };

        let run_client = async move {
            let result = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await;
            assert_eq!(result, Err(crate::Error::Timeout));
        };

        tokio::join!(run_server, run_client);
    }

    #[tokio::test]
    async fn recover_after_no_ack() {
        let (t1, mut t2) = TokioChannelTransport::new_pair(256);

        let mut client = Client::<_, Request, Response>::new(t1);

        let run_server = async move {
            // No ack
            let expected_request = RpcMessage {
                seq: 1,
                kind: RpcMessageKind::Request {
                    payload: Request::Ping(42),
                },
            };

            assert_eq!(
                t2.receive_message(Duration::from_millis(20)).await.unwrap(),
                expected_request
            );

            tokio::time::sleep(ACK_TIMEOUT).await;

            // Normal
            assert_eq!(
                t2.receive_message(Duration::from_millis(20)).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::Request {
                        payload: Request::Ping(42),
                    },
                }
            );

            t2.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::RequestAck,
            })
            .await
            .unwrap();

            t2.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::Response {
                    payload: Response::Ping(42),
                },
            })
            .await
            .unwrap();

            assert_eq!(
                t2.receive_message(Duration::ZERO).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::ResponseAck,
                }
            );

            // Exit when client goes away
            assert_eq!(
                t2.receive_message(Duration::from_secs(30)).await,
                Err(crate::Error::TransportError)
            );
        };

        let run_client = async move {
            // No ack
            let result = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await;
            assert_eq!(result, Err(crate::Error::NoAck));

            // Normal
            let result = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await;
            assert_eq!(result, Ok(Response::Ping(42)));
        };

        tokio::join!(run_server, run_client);
    }

    #[tokio::test]
    async fn recover_after_no_response() {
        let (t1, mut t2) = TokioChannelTransport::new_pair(256);

        let mut client = Client::<_, Request, Response>::new(t1);

        let run_server = async move {
            // No response
            assert_eq!(
                t2.receive_message(Duration::from_millis(20)).await.unwrap(),
                RpcMessage {
                    seq: 1,
                    kind: RpcMessageKind::Request {
                        payload: Request::Ping(42),
                    },
                }
            );

            t2.transmit_message(RpcMessage {
                seq: 1,
                kind: RpcMessageKind::RequestAck,
            })
            .await
            .unwrap();

            // Wait for the request to timeout
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Normal
            assert_eq!(
                t2.receive_message(Duration::from_millis(20)).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::Request {
                        payload: Request::Ping(42),
                    },
                }
            );

            t2.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::RequestAck,
            })
            .await
            .unwrap();

            t2.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::Response {
                    payload: Response::Ping(42),
                },
            })
            .await
            .unwrap();

            assert_eq!(
                t2.receive_message(Duration::ZERO).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::ResponseAck,
                }
            );

            // Exit when client goes away
            assert_eq!(
                t2.receive_message(Duration::from_secs(30)).await,
                Err(crate::Error::TransportError)
            );
        };

        let run_client = async move {
            // No response
            let result = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await;
            assert_eq!(result, Err(crate::Error::Timeout));

            // Normal
            let result = client
                .call(Request::Ping(42), Duration::from_millis(500))
                .await;
            assert_eq!(result, Ok(Response::Ping(42)));
        };

        tokio::join!(run_server, run_client);
    }
}
