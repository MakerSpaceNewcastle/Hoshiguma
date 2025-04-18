use crate::display::{
    drawables::{info_background::INFO_PANE_REGION, text::GenericText},
    DrawType, DrawTypeDrawable,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
};

pub(super) struct Home {}

impl DrawTypeDrawable for Home {
    type Color = Rgb565;
    type Output = ();

    fn draw<D>(&self, target: &mut D, draw_type: &DrawType) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let cursor = Point::new(
            INFO_PANE_REGION.top_left.x + 2,
            INFO_PANE_REGION.top_left.y + 11,
        );

        const TEXT: &str = "What are you looking\nat?\nThere is no infor-\nmation here.\n\nRead the wiki.\nIt has links for the\nGrafana dashboards.";
        GenericText::new(cursor, TEXT).draw(target, draw_type)?;

        Ok(())
    }
}
