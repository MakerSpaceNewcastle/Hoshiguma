pub(crate) struct StatusLamp {
    pub(crate) red: Lamp,
    pub(crate) amber: Lamp,
    pub(crate) green: Lamp,
}

impl StatusLamp {
    fn r#static(red: bool, amber: bool, green: bool) -> Self {
        Self {
            red: red.into(),
            amber: amber.into(),
            green: green.into(),
        }
    }

    fn red() -> Self {
        Self::r#static(true, false, false)
    }

    fn amber() -> Self {
        Self::r#static(false, true, false)
    }

    fn green() -> Self {
        Self::r#static(false, false, true)
    }
}

pub(crate) enum Lamp {
    On,
    Off,
}

impl From<bool> for Lamp {
    fn from(on: bool) -> Self {
        if on {
            Lamp::On
        } else {
            Lamp::Off
        }
    }
}
