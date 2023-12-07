use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, SwashCache, Shaping};

pub trait FontRasterizer {
    fn rasterize_text_line(&mut self, text: &str, font_size: f32) -> Vec<Vec<u8>>; 
}

struct CosmicFontRasterizer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl FontRasterizer for CosmicFontRasterizer {
    fn rasterize_text_line(&mut self, text: &str, font_size: f32) -> Vec<Vec<u8>> {
        let metrics = Metrics::new(font_size, font_size);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let mut buffer = buffer.borrow_with(&mut self.font_system);
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
    Box::new(CosmicFontRasterizer {
        font_system: FontSystem::new(),
        swash_cache: SwashCache::new(),
    })
}