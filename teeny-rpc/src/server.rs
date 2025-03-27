use crate::{debug, trace, transport::Transport, warn, RpcMessage, RpcMessageKind};
use core::{marker::PhantomData, time::Duration};
use serde::{Deserialize, Serialize};

pub struct Server<
    'de,
    T,
    REQ: Serialize + Deserialize<'de> + PartialEq,
    RESP: Serialize + Deserialize<'de> + PartialEq,
> where
    T: Transport<RpcMessage<REQ, RESP>>,
{
    transport: T,
    active_rpc: Option<u32>,
    _request: PhantomData<REQ>,
    _response: PhantomData<RESP>,
    _de_lifetime: PhantomData<&'de ()>,
}

impl<
        'de,
        T,
        REQ: Serialize + Deserialize<'de> + PartialEq + Clone,
        RESP: Serialize + Deserialize<'de> + PartialEq + Clone,
    > Server<'de, T, REQ, RESP>
where
    T: Transport<RpcMessage<REQ, RESP>>,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            active_rpc: None,
            _request: PhantomData,
            _response: PhantomData,
            _de_lifetime: PhantomData,
        }
    }

    pub async fn wait_for_request(&mut self, timeout: Duration) -> Result<REQ, crate::Error> {
        if self.active_rpc.is_some() {
            return Err(crate::Error::RequestAlreadyInProgress);
        }

        let request = self.transport.receive_message(timeout).await?;

        if let RpcMessageKind::Request { payload } = request.kind {
            debug!("Received request {}", request.seq);

            trace!("Sending ack for request {}", request.seq);
            self.transport
                .transmit_message(RpcMessage {
                    seq: request.seq,
                    kind: RpcMessageKind::RequestAck,
                })
                .await?;

            self.active_rpc = Some(request.seq);
            Ok(payload)
        } else {
            Err(crate::Error::IncorrectMessageType)
        }
    }

    pub async fn send_response(&mut self, response: RESP) -> Result<(), crate::Error> {
        // Leave no active RPC after any attempt to send a response (cannot recover it if
        // send_response fails anyway)
        if let Some(seq) = self.active_rpc.take() {
            let response = RpcMessage {
                seq,
                kind: RpcMessageKind::Response { payload: response },
            };

            // Send the response
            trace!("Sending response {}", seq,);
            self.transport.transmit_message(response.clone()).await?;

            // Receive the response acknowledgement
            match self.transport.receive_message(crate::ACK_TIMEOUT).await {
                Ok(ack) => {
                    if ack.kind != RpcMessageKind::ResponseAck {
                        return Err(crate::Error::IncorrectMessageType);
                    }
                    if ack.seq != seq {
                        return Err(crate::Error::IncorrectSequenceNumber {
                            expected: seq,
                            actual: ack.seq,
                        });
                    }

                    trace!("Received ack for response {}", seq);

                    debug!("Request {} complete", seq);
                    Ok(())
                }
                Err(crate::Error::Timeout) => {
                    warn!("Timeout waiting for ack for response {}", seq);
                    Err(crate::Error::NoAck)
                }
                Err(e) => {
                    warn!("Error waiting for ack for response {}: {}", seq, e);
                    Err(crate::Error::NoAck)
                }
            }
        } else {
            Err(crate::Error::NoRequestInProgress)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        server::Server,
        test::{Request, Response},
        transport::{tokio_channels::TokioChannelTransport, Transport},
        RpcMessage, RpcMessageKind, ACK_TIMEOUT,
    };
    use core::time::Duration;

    #[tokio::test]
    async fn basic() {
        let (mut t1, t2) = TokioChannelTransport::new_pair(256);

        let mut server = Server::<_, Request, Response>::new(t2);

        let run_server = {
            async move {
                loop {
                    let request = match server.wait_for_request(Duration::from_secs(5)).await {
                        Ok(request) => request,
                        Err(crate::Error::TransportError) => {
                            // Exit when the client goes away
                            break;
                        }
                        Err(e) => {
                            panic!("Server error: {e}");
                        }
                    };

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
            t1.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::Request {
                    payload: Request::Ping(69),
                },
            })
            .await
            .unwrap();

            let msg = t1.receive_message(Duration::ZERO).await.unwrap();
            assert_eq!(msg.seq, 2);
            assert_eq!(msg.kind, RpcMessageKind::RequestAck);

            let msg = t1.receive_message(Duration::ZERO).await.unwrap();
            assert_eq!(msg.seq, 2);
            assert_eq!(
                msg.kind,
                RpcMessageKind::Response {
                    payload: Response::Ping(69)
                }
            );

            t1.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::ResponseAck,
            })
            .await
            .unwrap();
        };

        tokio::join!(run_server, run_client);
    }

    #[tokio::test]
    async fn no_ack() {
        let (mut t1, t2) = TokioChannelTransport::new_pair(256);

        let mut server = Server::<_, Request, Response>::new(t2);

        let run_server = {
            async move {
                loop {
                    let request = match server.wait_for_request(Duration::from_secs(5)).await {
                        Ok(request) => request,
                        Err(crate::Error::TransportError) => {
                            // Exit when the client goes away
                            break;
                        }
                        Err(e) => {
                            panic!("Server error: {e}");
                        }
                    };

                    let result = server
                        .send_response(match request {
                            Request::Ping(i) => Response::Ping(i),
                        })
                        .await;
                    assert_eq!(result, Err(crate::Error::NoAck));
                }
            }
        };

        let run_client = async move {
            t1.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::Request {
                    payload: Request::Ping(69),
                },
            })
            .await
            .unwrap();

            assert_eq!(
                t1.receive_message(Duration::ZERO).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::RequestAck,
                }
            );

            assert_eq!(
                t1.receive_message(Duration::from_millis(20)).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::Response {
                        payload: Response::Ping(69)
                    }
                }
            );

            tokio::time::sleep(ACK_TIMEOUT).await;

            // Wait for the request to timeout
            tokio::time::sleep(Duration::from_millis(20)).await;
        };

        tokio::join!(run_server, run_client);
    }

    #[tokio::test]
    async fn recover_after_no_ack() {
        let (mut t1, t2) = TokioChannelTransport::new_pair(256);

        let mut server = Server::<_, Request, Response>::new(t2);

        let run_server = {
            async move {
                loop {
                    let request = match server.wait_for_request(Duration::from_secs(5)).await {
                        Ok(request) => request,
                        Err(crate::Error::TransportError) => {
                            // Exit when the client goes away
                            break;
                        }
                        Err(e) => {
                            panic!("Server error: {e}");
                        }
                    };

                    let _ = server
                        .send_response(match request {
                            Request::Ping(i) => Response::Ping(i),
                        })
                        .await;
                }
            }
        };

        let run_client = async move {
            t1.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::Request {
                    payload: Request::Ping(69),
                },
            })
            .await
            .unwrap();

            assert_eq!(
                t1.receive_message(Duration::ZERO).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::RequestAck,
                }
            );

            assert_eq!(
                t1.receive_message(Duration::from_millis(20)).await.unwrap(),
                RpcMessage {
                    seq: 2,
                    kind: RpcMessageKind::Response {
                        payload: Response::Ping(69)
                    }
                }
            );

            tokio::time::sleep(ACK_TIMEOUT).await;

            // Wait for the request to timeout
            tokio::time::sleep(Duration::from_millis(20)).await;

            t1.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::Request {
                    payload: Request::Ping(69),
                },
            })
            .await
            .unwrap();

            let msg = t1.receive_message(Duration::ZERO).await.unwrap();
            assert_eq!(msg.seq, 2);
            assert_eq!(msg.kind, RpcMessageKind::RequestAck);

            let msg = t1.receive_message(Duration::ZERO).await.unwrap();
            assert_eq!(msg.seq, 2);
            assert_eq!(
                msg.kind,
                RpcMessageKind::Response {
                    payload: Response::Ping(69)
                }
            );

            t1.transmit_message(RpcMessage {
                seq: 2,
                kind: RpcMessageKind::ResponseAck,
            })
            .await
            .unwrap();
        };

        tokio::join!(run_server, run_client);
    }
}
