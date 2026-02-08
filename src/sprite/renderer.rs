use image::{DynamicImage, GenericImageView};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

/// A rendered sprite: rows of (fg_color, bg_color, char) cells using half-block unicode
pub struct SpriteWidget {
    /// Each row is a Vec of (char, fg, bg) — one terminal row = 2 pixel rows via ▄/▀
    cells: Vec<Vec<(char, Color, Color)>>,
}

impl SpriteWidget {
    pub fn from_png_bytes(bytes: &[u8], max_width: u16, max_height: u16) -> Option<Self> {
        let img = image::load_from_memory(bytes).ok()?;
        Some(Self::from_image(&img, max_width, max_height))
    }

    pub fn from_image(img: &DynamicImage, max_width: u16, max_height: u16) -> Self {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        // Scale to fit — each terminal cell is 1 char wide, 2 pixels tall (half-block)
        let scale_x = (w as f64) / (max_width as f64);
        let scale_y = (h as f64) / ((max_height * 2) as f64);
        let scale = scale_x.max(scale_y).max(1.0);

        let out_w = ((w as f64) / scale).ceil() as u32;
        let out_h = ((h as f64) / scale).ceil() as u32;
        // Make sure height is even for half-block pairing
        // Using manual modulo check instead of is_multiple_of() for stable Rust compatibility
        #[allow(clippy::manual_is_multiple_of)]
        let out_h = if out_h % 2 != 0 { out_h + 1 } else { out_h };

        let mut cells = Vec::new();

        for row in (0..out_h).step_by(2) {
            let mut line = Vec::new();
            for col in 0..out_w {
                let top = sample_pixel(img, col, row, scale);
                let bot = sample_pixel(img, col, row + 1, scale);

                let (ch, fg, bg) = match (top, bot) {
                    (None, None) => (' ', Color::Reset, Color::Reset),
                    (Some(tc), None) => ('▀', to_color(tc), Color::Reset),
                    (None, Some(bc)) => ('▄', to_color(bc), Color::Reset),
                    (Some(tc), Some(bc)) => ('▀', to_color(tc), to_color(bc)),
                };
                line.push((ch, fg, bg));
            }
            cells.push(line);
        }

        SpriteWidget { cells }
    }
}

fn sample_pixel(img: &DynamicImage, col: u32, row: u32, scale: f64) -> Option<(u8, u8, u8)> {
    let sx = ((col as f64) * scale) as u32;
    let sy = ((row as f64) * scale) as u32;
    let (w, h) = img.dimensions();
    if sx >= w || sy >= h {
        return None;
    }
    let pixel = img.get_pixel(sx, sy);
    if pixel[3] < 128 {
        None // transparent
    } else {
        Some((pixel[0], pixel[1], pixel[2]))
    }
}

fn to_color(rgb: (u8, u8, u8)) -> Color {
    Color::Rgb(rgb.0, rgb.1, rgb.2)
}

impl Widget for &SpriteWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (row_idx, row) in self.cells.iter().enumerate() {
            if row_idx as u16 >= area.height {
                break;
            }
            for (col_idx, &(ch, fg, bg)) in row.iter().enumerate() {
                if col_idx as u16 >= area.width {
                    break;
                }
                let x = area.x + col_idx as u16;
                let y = area.y + row_idx as u16;
                buf[(x, y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(fg).bg(bg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = if (x + y) % 2 == 0 {
                    Rgba([255, 0, 0, 255]) // Red
                } else {
                    Rgba([0, 0, 255, 255]) // Blue
                };
                img.put_pixel(x, y, pixel);
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    fn create_transparent_image(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0])); // Transparent
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_sprite_widget_from_image() {
        let img = create_test_image(4, 4);
        let widget = SpriteWidget::from_image(&img, 10, 10);

        // Should create cells (height/2 rows since we pair pixels)
        assert!(!widget.cells.is_empty());
    }

    #[test]
    fn test_sprite_widget_from_image_scaling() {
        let img = create_test_image(20, 20);
        let widget = SpriteWidget::from_image(&img, 10, 10);

        // Should scale down to fit max dimensions
        assert!(widget.cells.len() <= 10);
        if !widget.cells.is_empty() {
            assert!(widget.cells[0].len() <= 10);
        }
    }

    #[test]
    fn test_sprite_widget_from_png_bytes() {
        // Test with invalid bytes first
        let invalid_bytes = b"not a png";
        let widget = SpriteWidget::from_png_bytes(invalid_bytes, 10, 10);
        assert!(widget.is_none());

        // Note: Testing with actual PNG encoding would require more complex setup
        // The from_image method is tested separately, which covers the core logic
    }

    #[test]
    fn test_to_color() {
        let color = super::to_color((255, 128, 64));
        match color {
            Color::Rgb(r, g, b) => {
                assert_eq!(r, 255);
                assert_eq!(g, 128);
                assert_eq!(b, 64);
            }
            _ => panic!("Expected Rgb color"),
        }
    }

    #[test]
    fn test_sample_pixel_within_bounds() {
        let img = create_test_image(10, 10);
        let pixel = super::sample_pixel(&img, 5, 5, 1.0);
        assert!(pixel.is_some());
    }

    #[test]
    fn test_sample_pixel_out_of_bounds() {
        let img = create_test_image(10, 10);
        let pixel = super::sample_pixel(&img, 20, 20, 1.0);
        assert!(pixel.is_none());
    }

    #[test]
    fn test_sample_pixel_transparent() {
        let img = create_transparent_image(10, 10);
        let pixel = super::sample_pixel(&img, 5, 5, 1.0);
        assert!(pixel.is_none()); // Transparent pixels return None
    }

    #[test]
    fn test_sprite_widget_from_image_single_pixel() {
        let img = create_test_image(1, 1);
        let widget = SpriteWidget::from_image(&img, 10, 10);
        // Should handle small images gracefully - should create at least one cell row
        // (height is made even, so 1 pixel becomes 2, which creates 1 row)
        assert!(!widget.cells.is_empty());
        assert_eq!(widget.cells.len(), 1);
        // Verify the cell contains actual data (not empty)
        assert!(!widget.cells[0].is_empty());
    }
}
