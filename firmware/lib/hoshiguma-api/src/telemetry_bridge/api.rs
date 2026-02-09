pub mod request {
    crate::define_message!(IsReady, (), b"tlm/t/q/rd");
    crate::define_request_response!(IsReady, super::response::Ready);

    crate::define_message!(GetTime, (), b"tlm/t/q/tm");
    crate::define_request_response!(GetTime, super::response::Time);

    crate::define_message!(
        SendTelemetryDataPoint,
        (pub super::super::FormattedTelemetryDataPoint),
        b"rsb/t/q/dp"
    );
    crate::define_request_response!(
        SendTelemetryDataPoint,
        super::response::TelemetryDataPointAck
    );
}

pub mod response {
    use chrono::{DateTime, Utc};

    crate::define_message!(ApiError, (), b"tlm/t/p/ae");

    crate::define_message!(Ready, (pub bool), b"tlm/t/r/rd");

    crate::define_message!(Time, (pub Option<DateTime<Utc>>), b"tlm/t/r/tm");

    crate::define_message!(TelemetryDataPointAck, (), b"tlm/t/r/dp");
}
