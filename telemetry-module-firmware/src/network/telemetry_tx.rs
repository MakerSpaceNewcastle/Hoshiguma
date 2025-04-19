use heapless::String;

pub(super) struct TelemetryBuffer {
    body: String<1024>,
}
