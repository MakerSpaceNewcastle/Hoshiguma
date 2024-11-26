use crate::display::{screens::ScreenSelector, SCREEN_WIDTH};
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point, Primitive, Size, WebColors},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
    Drawable,
};

pub(super) const HEIGHT: u32 = 12;
const REGION: Rectangle = Rectangle::new(Point::new(0, 0), Size::new(SCREEN_WIDTH as u32, HEIGHT));
const BACKGROUND_COLOUR: Rgb565 = Rgb565::CSS_DARK_SLATE_GRAY;

pub(crate) struct TitleBar {
    screen_number: usize,
    screen_title: &'static str,
}

impl TitleBar {
    pub(crate) fn new(screen_selector: &ScreenSelector) -> Self {
        Self {
            screen_number: screen_selector.current_screen_number(),
            screen_title: screen_selector.current_screen().name(),
        }
    }
}

impl Drawable for TitleBar {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // Draw the title bar background
        let background_style = PrimitiveStyleBuilder::new()
            .fill_color(BACKGROUND_COLOUR)
            .build();
        REGION.into_styled(background_style).draw(target)?;

        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::CSS_WHITE);

        let mut s = heapless::String::<32>::new();
        s.write_fmt(format_args!(
            "{}/{} {}",
            self.screen_number,
            ScreenSelector::num_screens(),
            self.screen_title
        ))
        .expect("text buffer should be large enough for the longest possible title bar text");

        let p = Point::new(REGION.top_left.x + 2, REGION.bottom_right().unwrap().y - 3);
        Text::with_alignment(&s, p, text_style, Alignment::Left).draw(target)?;

        Ok(())
    }
}
