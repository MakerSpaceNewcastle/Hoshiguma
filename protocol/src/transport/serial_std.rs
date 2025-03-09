pub struct Server<R, S> {}

impl<R, S> super::Server<R, S> for Server<R, S> {
    async fn receive_message(
        &mut self,
        timeout: core::time::Duration,
    ) -> Result<crate::Message<R, S>, ()> {
        todo!()
    }

    async fn send_stream_message(&mut self, data: S) -> Result<(), ()> {
        todo!()
    }
}

pub struct Client<R, S> {}

impl<R, S> super::Client<R, S> for Client<R, S> {
    async fn receive_message(
        &mut self,
        timeout: core::time::Duration,
    ) -> Result<crate::Message<R, S>, ()> {
        todo!()
    }

    async fn send_rpc_message(&mut self, msg: R) -> Result<R, ()> {
        todo!()
    }
}
