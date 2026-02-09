pub mod request {
    crate::define_message!(GetSystemInformation, (), b"clr/t/q/si");
    crate::define_request_response!(GetSystemInformation, super::response::SystemInformation);

    crate::define_message!(SetRadiatorFanState, (pub crate::cooler::RadiatorFanState), b"clr/t/q/rf");
    crate::define_request_response!(SetRadiatorFanState, super::response::RadiatorFanState);
    crate::falible_basic_state_response_verification!(
        SetRadiatorFanState,
        super::response::RadiatorFanState
    );

    crate::define_message!(SetCompressorState, (pub crate::cooler::CompressorState), b"clr/t/q/cm");
    crate::define_request_response!(SetCompressorState, super::response::CompressorState);
    crate::falible_basic_state_response_verification!(
        SetCompressorState,
        super::response::CompressorState
    );

    crate::define_message!(SetCoolantPumpState, (pub crate::cooler::CoolantPumpState), b"clr/t/q/cp");
    crate::define_request_response!(SetCoolantPumpState, super::response::CoolantPumpState);
    crate::falible_basic_state_response_verification!(
        SetCoolantPumpState,
        super::response::CoolantPumpState
    );

    crate::define_message!(GetTemperatures, (), b"clr/t/q/tp");
    crate::define_request_response!(GetTemperatures, super::response::Temperatures);

    crate::define_message!(GetCoolantFlowRate, (), b"clr/t/q/cf");
    crate::define_request_response!(GetCoolantFlowRate, super::response::CoolantFlowRate);

    crate::define_message!(GetCoolantFlowPulses, (), b"clr/t/q/pf");
    crate::define_request_response!(GetCoolantFlowPulses, super::response::CoolantFlowPulses);

    crate::define_message!(GetCoolantReturnRate, (), b"clr/t/q/cr");
    crate::define_request_response!(GetCoolantReturnRate, super::response::CoolantReturnRate);

    crate::define_message!(GetCoolantReturnPulses, (), b"clr/t/q/pr");
    crate::define_request_response!(GetCoolantReturnPulses, super::response::CoolantReturnPulses);
}

pub mod response {
    crate::define_message!(ApiError, (), b"clr/t/p/ae");

    crate::define_message!(SystemInformation, (pub crate::types::SystemInformation), b"clr/t/p/si");

    impl From<SystemInformation> for crate::types::SystemInformation {
        fn from(info: SystemInformation) -> Self {
            info.0
        }
    }

    crate::define_message!(RadiatorFanState, (pub Result<super::super::types::RadiatorFanState, ()>), b"clr/t/p/rf");

    crate::define_message!(CompressorState, (pub Result<super::super::types::CompressorState, ()>), b"clr/t/p/cm");

    crate::define_message!(CoolantPumpState, (pub Result<super::super::types::CoolantPumpState, ()>), b"clr/t/p/cp");

    crate::define_message!(Temperatures, (pub Result<crate::OnewireTemperatureSensorReadings, ()>), b"clr/t/p/tp");

    crate::define_message!(CoolantFlowRate, (pub Result<crate::cooler::RawCoolantRate, ()>), b"clr/t/p/cf");
    crate::define_message!(CoolantFlowPulses, (pub Result<u64, ()>), b"clr/t/p/pf");

    crate::define_message!(CoolantReturnRate, (pub Result<crate::cooler::RawCoolantRate, ()>), b"clr/t/p/cr");
    crate::define_message!(CoolantReturnPulses, (pub Result<u64, ()>), b"clr/t/p/pr");
}
