pub mod request {
    crate::define_message!(NotifyPanelInteraction, (), b"hmi/f/q/pi");
    crate::define_request_response!(NotifyPanelInteraction, super::response::AckPanelInteraction);
    impl crate::ResponseVerification<super::response::AckPanelInteraction> for NotifyPanelInteraction {
        fn verify_response(&self, _: &super::response::AckPanelInteraction) -> bool {
            true
        }
    }

    crate::define_message!(NotifyAccessControlInputChanged, (pub super::super::super::AccessControlRawInput), b"hmi/f/q/ai");
    crate::define_request_response!(
        NotifyAccessControlInputChanged,
        super::response::AckAccessControlInputChanged
    );
    crate::basic_state_response_verification!(
        NotifyAccessControlInputChanged,
        super::response::AckAccessControlInputChanged
    );

    crate::define_message!(NotifyAccessControlStateChanged, (pub super::super::super::AccessControlState), b"hmi/f/q/as");
    crate::define_request_response!(
        NotifyAccessControlStateChanged,
        super::response::AckAccessControlStateChanged
    );
    crate::basic_state_response_verification!(
        NotifyAccessControlStateChanged,
        super::response::AckAccessControlStateChanged
    );
}

pub mod response {
    crate::define_message!(ApiError, (), b"hmi/f/p/ae");

    crate::define_message!(AckPanelInteraction, (), b"hmi/f/p/pi");

    crate::define_message!(AckAccessControlInputChanged, (pub super::super::super::AccessControlRawInput), b"hmi/f/p/ai");

    crate::define_message!(AckAccessControlStateChanged, (pub super::super::super::AccessControlState), b"hmi/f/p/as");
}
