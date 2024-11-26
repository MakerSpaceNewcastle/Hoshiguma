pub(crate) mod boot_screen;
pub(crate) mod info_pane_background;
pub(crate) mod measurement;
pub(crate) mod subtitle;
pub(crate) mod title_bar;

use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors};

const LIGHT_TEXT_COLOUR: Rgb565 = Rgb565::CSS_MOCCASIN;
