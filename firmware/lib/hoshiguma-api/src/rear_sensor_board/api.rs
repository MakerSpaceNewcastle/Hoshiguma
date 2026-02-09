pub mod request {
    crate::define_message!(GetSystemInformation, (), b"rsb/t/q/si");
    crate::define_request_response!(GetSystemInformation, super::response::SystemInformation);

    crate::define_message!(SetStatusLight, (pub super::super::types::StatusLightSettings), b"rsb/t/q/sl");
    crate::define_request_response!(SetStatusLight, super::response::StatusLightSettings);
    crate::falible_basic_state_response_verification!(
        SetStatusLight,
        super::response::StatusLightSettings
    );

    crate::define_message!(GetExtractionAirflow, (), b"rsb/t/q/ea");
    crate::define_request_response!(GetExtractionAirflow, super::response::ExtractionAirflow);

    crate::define_message!(GetTemperatures, (), b"rsb/t/q/tp");
    crate::define_request_response!(GetTemperatures, super::response::Temperatures);
}

pub mod response {
    crate::define_message!(ApiError, (), b"rsb/t/p/ae");

    crate::define_message!(SystemInformation, (pub crate::SystemInformation), b"rsb/t/r/si");

    impl From<SystemInformation> for crate::types::SystemInformation {
        fn from(info: SystemInformation) -> Self {
            info.0
        }
    }

    crate::define_message!(StatusLightSettings, (pub Result<super::super::types::StatusLightSettings, ()>), b"rsb/t/r/sl");

    crate::define_message!(ExtractionAirflow, (pub Result<crate::AirflowSensorMeasurement, ()>), b"rsb/t/r/ea");

    crate::define_message!(Temperatures, (pub Result<crate::OnewireTemperatureSensorReadings, ()>), b"rsb/t/r/tp");
}
