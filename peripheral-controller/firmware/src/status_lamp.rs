use embassy_rp::gpio::{Level, Output};

pub(crate) struct StatusLamp {
    red: Output<'static>,
    amber: Output<'static>,
    green: Output<'static>,
}

impl StatusLamp {
    pub(crate) fn new(
        red: Output<'static>,
        amber: Output<'static>,
        green: Output<'static>,
    ) -> Self {
        let mut s = Self { red, amber, green };
        s.output(&StatusLampSetting::red());
        s
    }

    pub(crate) fn output(&mut self, settings: &StatusLampSetting) {
        fn level_for_lamp(l: &LampSetting) -> Level {
            match l {
                LampSetting::On => Level::High,
                LampSetting::Off => Level::Low,
            }
        }

        self.red.set_level(level_for_lamp(&settings.red));
        self.amber.set_level(level_for_lamp(&settings.amber));
        self.green.set_level(level_for_lamp(&settings.green));
    }
}

pub(crate) struct StatusLampSetting {
    pub(crate) red: LampSetting,
    pub(crate) amber: LampSetting,
    pub(crate) green: LampSetting,
}

impl StatusLampSetting {
    fn r#static(red: bool, amber: bool, green: bool) -> Self {
        Self {
            red: red.into(),
            amber: amber.into(),
            green: green.into(),
        }
    }

    pub(crate) fn red() -> Self {
        Self::r#static(true, false, false)
    }

    // pub(crate) fn amber() -> Self {
    //     Self::r#static(false, true, false)
    // }

    // pub(crate) fn green() -> Self {
    //     Self::r#static(false, false, true)
    // }
}

pub(crate) enum LampSetting {
    On,
    Off,
}

impl From<bool> for LampSetting {
    fn from(on: bool) -> Self {
        if on {
            LampSetting::On
        } else {
            LampSetting::Off
        }
    }
}
