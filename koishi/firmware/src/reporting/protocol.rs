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
