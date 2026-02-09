pub mod request {
    crate::define_message!(GetSystemInformation, (), b"hmi/t/q/si");
    crate::define_request_response!(GetSystemInformation, super::response::SystemInformation);

    crate::define_message!(SetBacklight, (pub super::super::super::BacklightMode), b"hmi/t/q/bm");
    crate::define_request_response!(SetBacklight, super::response::BacklightMode);
    crate::falible_basic_state_response_verification!(SetBacklight, super::response::BacklightMode);

    crate::define_message!(BacklightWake, (), b"hmi/t/q/bw");
    crate::define_request_response!(BacklightWake, super::response::AckBacklightWake);

    crate::define_message!(ShowScreen, (pub super::super::super::Screen), b"hmi/t/q/sc");
    crate::define_request_response!(ShowScreen, super::response::ActiveScreen);

    crate::define_message!(SetStatusScreenInfo, (pub super::super::super::StatusScreenInfo), b"hmi/t/q/ps");
    crate::define_request_response!(SetStatusScreenInfo, super::response::AckStatusScreenInfo);
}

pub mod response {
    crate::define_message!(ApiError, (), b"hmi/t/p/ae");

    crate::define_message!(SystemInformation, (pub crate::types::SystemInformation), b"hmi/t/p/si");

    impl From<SystemInformation> for crate::types::SystemInformation {
        fn from(info: SystemInformation) -> Self {
            info.0
        }
    }

    crate::define_message!(BacklightMode, (pub Result<super::super::super::BacklightMode, ()>), b"hmi/t/p/bm");

    crate::define_message!(AckBacklightWake, (), b"hmi/t/p/bw");

    crate::define_message!(ActiveScreen, (pub super::super::super::Screen), b"hmi/t/p/sc");

    crate::define_message!(AckStatusScreenInfo, (), b"hmi/t/p/ps");
}
