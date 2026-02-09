use crate::{
    changed::{Changed, ObservedValue},
    network::send_request,
};
use core::{marker::PhantomData, net::Ipv4Addr};
use defmt::{debug, info, warn};
use embassy_net::Stack;
use hoshiguma_api::{API_PORT, ExpectedResponse, MessagePayload, ResponseVerification};
use serde::{Serialize, de::DeserializeOwned};

pub struct RemoteStateReconciler<ReqT: Clone + PartialEq, RespT> {
    net_stack: Stack<'static>,
    device_ip: Ipv4Addr,

    desired_state: ObservedValue<ReqT>,

    _api_types: PhantomData<(ReqT, RespT)>,
}

impl<ReqT, RespT> RemoteStateReconciler<ReqT, RespT>
where
    ReqT: ExpectedResponse<Response = RespT>
        + MessagePayload
        + Serialize
        + ResponseVerification<RespT>
        + Clone
        + PartialEq,
    RespT: MessagePayload + DeserializeOwned,
{
    pub fn new(net_stack: Stack<'static>, device_ip: Ipv4Addr) -> Self {
        Self {
            net_stack,
            device_ip,
            desired_state: ObservedValue::default(),
            _api_types: PhantomData,
        }
    }

    pub fn set_desired_state(&mut self, request: ReqT) -> Changed {
        self.desired_state.update(request)
    }

    pub async fn reconcile(&mut self) -> Result<Option<RespT>, ()> {
        if let Some(desired_state) = &*self.desired_state {
            match send_request(self.net_stack, self.device_ip, API_PORT, 1, desired_state).await {
                Ok(response) => {
                    if desired_state.verify_response(&response) {
                        debug!("Remote state is correct");
                        Ok(Some(response))
                    } else {
                        warn!("Received invalid response");
                        Err(())
                    }
                }
                Err(e) => {
                    warn!("Failed to reconcile: {}", e);
                    Err(())
                }
            }
        } else {
            info!("No desired state");
            Ok(None)
        }
    }
}
