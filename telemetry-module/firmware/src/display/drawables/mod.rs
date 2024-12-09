pub(crate) mod boot_screen;
pub(crate) mod measurement;
pub(crate) mod screen;
pub(crate) mod subtitle;

use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors};

const LIGHT_TEXT_COLOUR: Rgb565 = Rgb565::CSS_MOCCASIN;
