use std::path::PathBuf;

use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, SwashCache, Shaping, fontdb::{Source, Database}};

pub trait FontRasterizer {
    fn rasterize_text_line(&mut self, font: &Font, text: &str, font_size: f32) -> Vec<Vec<u8>>; 
}

#[derive(Debug)]
pub enum Font {
    TekoRegular
}

struct CosmicFontRasterizer {
    teko_regular_font: FontSystem,
    swash_cache: SwashCache,
}

impl FontRasterizer for CosmicFontRasterizer {
    fn rasterize_text_line(&mut self, font: &Font, text: &str, font_size: f32) -> Vec<Vec<u8>> {
        if font_size <= 0.0 {
            // cosmic text panics if the font size is 0, so we might as well check beforehand
            panic!("rasterize_text_line(text={}) called with nonpositive font_size({})", text, font_size);
        }
        let metrics = Metrics::new(font_size, font_size);
        let font_system = match font {
            Font::TekoRegular => &mut self.teko_regular_font,
        };
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        let attrs = Attrs::new();
        buffer.set_size(f32::MAX, f32::MAX);
        buffer.set_text(text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(); // do we need this call?

        // rasterize buffer on canvas
        let mut canvas = vec![vec![0u8; 0]; 0];
        let color = Color::rgb(0, 0, 0); // Color doesn't matter, because we only care about alpha, not rgb
        buffer.draw(&mut self.swash_cache, color, |x, y, w, h, color| {
            // are these checks necessary?
            if x < 0 || y < 0 || w != 1 || h != 1 {
                return;
            }
            let x = x as usize;
            let y = y as usize;
            if canvas.len() <= y {
                canvas.resize(y + 1, Vec::new());
            }
            if canvas[y].len() <= x {
                canvas[y].resize(x + 1, 0u8);
            }
            canvas[y][x] = color.a();
        });
        if canvas.is_empty() {
            return canvas;
        }
        let max_width = canvas.iter().map(|row| row.len()).max().unwrap();
        canvas.iter_mut().for_each(|row| row.resize(max_width, 0u8));
        canvas
    }
}

impl CosmicFontRasterizer {

}

pub fn make_font_rasterizer() -> Box<dyn FontRasterizer> {
    // I'm not sure what locale is used for, but Cosmic Text requires it. I hardcode en-US so the experience is 
    // consistent across different devices.
    let mut teko_regular_font = FontSystem::new_with_locale_and_db(String::from("en-US"), Database::new());
    let id = teko_regular_font.db_mut().load_font_source(Source::File(PathBuf::from(r"fonts\Teko-Regular.ttf")));
    assert!(!id.is_empty(), "Teko Regular id is empty, so loading likely failed");

    Box::new(CosmicFontRasterizer {
        teko_regular_font,
        swash_cache: SwashCache::new(),
    })
}