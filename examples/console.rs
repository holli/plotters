use plotters::drawing::{
    backend::{BackendStyle, DrawingErrorKind},
    DrawingBackend,
};
use plotters::prelude::*;
use plotters::style::RGBAColor;
use std::error::Error;

#[derive(Copy, Clone)]
enum PixelState {
    Empty,
    HLine,
    VLine,
    Cross,
    Pixel,
    Text(char),
    Circle(bool),
}

impl PixelState {
    fn to_char(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::HLine => '-',
            Self::VLine => '|',
            Self::Cross => '+',
            Self::Pixel => '.',
            Self::Text(c) => *c,
            Self::Circle(filled) => {
                if *filled {
                    '@'
                } else {
                    'O'
                }
            }
        }
    }
    fn update(&mut self, new_state: PixelState) {
        let next_state = match (*self, new_state) {
            (Self::HLine, Self::VLine) => Self::Cross,
            (Self::VLine, Self::HLine) => Self::Cross,
            (_, Self::Circle(what)) => Self::Circle(what),
            (Self::Circle(what), _) => Self::Circle(what),
            (_, Self::Pixel) => Self::Pixel,
            (Self::Pixel, _) => Self::Pixel,
            (_, new) => new,
        };

        *self = next_state;
    }
}

pub struct TextDrawingBackend(Vec<PixelState>);

impl DrawingBackend for TextDrawingBackend {
    type ErrorType = std::io::Error;

    fn get_size(&self) -> (u32, u32) {
        (100, 30)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        for r in 0..30 {
            let mut buf = String::new();
            for c in 0..100 {
                buf.push(self.0[r * 100 + c].to_char());
            }
            println!("{}", buf);
        }

        Ok(())
    }

    fn draw_pixel(
        &mut self,
        pos: (i32, i32),
        color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<std::io::Error>> {
        if color.alpha() > 0.3 {
            self.0[(pos.1 * 100 + pos.0) as usize].update(PixelState::Pixel);
        }
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if from.0 == to.0 {
            let x = from.0;
            let y0 = from.1.min(to.1);
            let y1 = from.1.max(to.1);
            for y in y0..y1 {
                self.0[(y * 100 + x) as usize].update(PixelState::VLine);
            }
            return Ok(());
        }

        if from.1 == to.1 {
            let y = from.1;
            let x0 = from.0.min(to.0);
            let x1 = from.0.max(to.0);
            for x in x0..x1 {
                self.0[(y * 100 + x) as usize].update(PixelState::HLine);
            }
            return Ok(());
        }

        plotters::drawing::rasterizer::draw_line(self, from, to, style)
    }

    fn estimate_text_size<'a>(
        &self,
        text: &str,
        _font: &FontDesc<'a>,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        Ok((text.len() as u32, 1))
    }

    fn draw_text<'a>(
        &mut self,
        text: &str,
        _font: &FontDesc<'a>,
        pos: (i32, i32),
        _color: &RGBAColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let offset = pos.1.max(0) * 100 + pos.0.max(0);
        for (idx, chr) in (offset..).zip(text.chars()) {
            self.0[idx as usize].update(PixelState::Text(chr));
        }
        Ok(())
    }
}

fn draw_chart<DB: DrawingBackend>(
    b: DrawingArea<DB, plotters::coord::Shift>,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    let mut chart = ChartBuilder::on(&b)
        .margin(1)
        .caption("Sine and Cosine", ("sans-serif", (10).percent_height()))
        .set_label_area_size(LabelAreaPosition::Left, (5i32).percent_width())
        .set_label_area_size(LabelAreaPosition::Bottom, (10i32).percent_height())
        .set_label_area_size(LabelAreaPosition::Bottom, (10i32).percent_height())
        .build_ranged(-3.14..3.14, -1.2..1.2)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    chart.draw_series(LineSeries::new(
        (-314..314).map(|x| x as f64 / 100.0).map(|x| (x, x.sin())),
        &RED,
    ))?;

    chart.draw_series(LineSeries::new(
        (-314..314).map(|x| x as f64 / 100.0).map(|x| (x, x.cos())),
        &RED,
    ))?;

    b.present()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    draw_chart(TextDrawingBackend(vec![PixelState::Empty; 5000]).into_drawing_area())?;
    let b = BitMapBackend::new("plotters-doc-data/console-example.png", (1024, 768))
        .into_drawing_area();
    b.fill(&WHITE)?;
    draw_chart(b)?;
    Ok(())
}
