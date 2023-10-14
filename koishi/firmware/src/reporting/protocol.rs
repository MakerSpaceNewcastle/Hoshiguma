use crate::hal::TimeMillis;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct Message {
    time: TimeMillis,
    iteration_id: Option<u32>,
    payload: Payload,
}

impl Message {
    pub(super) fn new(iteration_id: Option<u32>, payload: Payload) -> Self {
        Self {
            time: crate::hal::millis(),
            iteration_id,
            payload,
        }
    }
}

#[derive(Serialize)]
pub(crate) enum Payload {
    Boot(BootPayload),
    Panic(PanicPayload),

    InputsChanged(crate::io::inputs::Inputs),
    OutputsChanged(crate::io::outputs::Outputs),

    MachineStatusChanged(crate::logic::machine::MachineStatus),
    AirAssistStatusChanged(crate::logic::air_assist::AirAssistStatus),
    ExtractionStatusChanged(crate::logic::extraction::ExtractionStatus),
}

impl From<&crate::io::inputs::Inputs> for Payload {
    fn from(inputs: &crate::io::inputs::Inputs) -> Payload {
        Payload::InputsChanged(inputs.clone())
    }
}

impl From<&crate::io::outputs::Outputs> for Payload {
    fn from(outputs: &crate::io::outputs::Outputs) -> Payload {
        Payload::OutputsChanged(outputs.clone())
    }
}

impl From<&crate::logic::machine::MachineStatus> for Payload {
    fn from(status: &crate::logic::machine::MachineStatus) -> Payload {
        Payload::MachineStatusChanged(status.clone())
    }
}

impl From<&crate::logic::air_assist::AirAssistStatus> for Payload {
    fn from(status: &crate::logic::air_assist::AirAssistStatus) -> Payload {
        Payload::AirAssistStatusChanged(status.clone())
    }
}

impl From<&crate::logic::extraction::ExtractionStatus> for Payload {
    fn from(status: &crate::logic::extraction::ExtractionStatus) -> Payload {
        Payload::ExtractionStatusChanged(status.clone())
    }
}

#[derive(Serialize)]
pub(crate) struct BootPayload {
    name: &'static str,
    git_revision: &'static str,
}

impl ufmt::uDebug for BootPayload {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uwrite!(f, "{} {}", self.name, self.git_revision)
    }
}

impl Default for BootPayload {
    fn default() -> Self {
        Self {
            name: "koishi",
            git_revision: git_version::git_version!(),
        }
    }
}

#[derive(Default, Serialize)]
pub(crate) struct PanicPayload {
    file: Option<heapless::String<32>>,
    line: Option<u32>,
    column: Option<u32>,
}

impl ufmt::uDebug for PanicPayload {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        if self.file.is_some() && self.line.is_some() && self.column.is_some() {
            ufmt::uwrite!(
                f,
                "at {}:{}:{}",
                self.file.as_ref().unwrap().as_str(),
                self.line.unwrap(),
                self.column.unwrap(),
            )
        } else {
            ufmt::uwrite!(f, "at unknown location")
        }
    }
}

impl From<&core::panic::PanicInfo<'_>> for PanicPayload {
    fn from(info: &core::panic::PanicInfo) -> Self {
        match info.location() {
            None => PanicPayload::default(),
            Some(loc) => Self {
                file: Some(loc.file().into()),
                line: Some(loc.line()),
                column: Some(loc.column()),
            },
        }
    }
}
