pub(crate) mod boot_screen;
pub(crate) mod diagnostics;
pub(crate) mod measurement;

use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors};

pub(crate) const BACKGROUND_COLOUR: Rgb565 = Rgb565::CSS_BLACK;
