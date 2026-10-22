use super::{
    canvas::Canvas,
    font5x7::{glyph, GLYPH_HEIGHT, GLYPH_SPACING, GLYPH_WIDTH}
};


impl<'a> Canvas<'a> {
    pub fn char(
        &mut self,
        x: i32,
        y: i32,
        character: char,
        color: u8,
    ) {
        if let Some(bitmap) = glyph(character) {
            for (row, bits) in bitmap.iter().enumerate() {
                for col in 0..GLYPH_WIDTH {
                    let mask = 1 << (GLYPH_WIDTH - 1 - col);

                    if bits & mask != 0 {
                        let px = x + col;
                        let py = y + row as i32;

                        if px >= 0 && py >= 0 {
                            self.framebuffer.set_pixel(
                                px as u32,
                                py as u32,
                                color,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn text(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        color: u8,
    ) {
        let mut cursor_x = x;

        for character in text.chars() {
            self.char(cursor_x, y, character, color);
            cursor_x += GLYPH_WIDTH + GLYPH_SPACING;
        }
    }

    pub fn text_scaled(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        color: u8,
        scale: u32,
    ) {
        if scale == 0 {
            return;
        }

        let mut cursor_x = x;

        for character in text.chars() {
            self.char_scaled(cursor_x, y, character, color, scale);
            cursor_x += (GLYPH_WIDTH + GLYPH_SPACING) * scale as i32;
        }
    }

    pub fn char_scaled(
        &mut self,
        x: i32,
        y: i32,
        character: char,
        color: u8,
        scale: u32,
    ) {
        if scale == 0 {
            return;
        }

        if let Some(bitmap) = glyph(character) {
            for (row, bits) in bitmap.iter().enumerate() {
                for col in 0..GLYPH_WIDTH {
                    let mask = 1 << (GLYPH_WIDTH - 1 - col);

                    if bits & mask != 0 {
                        self.fill_rect(
                            x + col * scale as i32,
                            y + row as i32 * scale as i32,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
    }

    pub fn text_width(text: &str) -> i32 {
        text.chars().count() as i32 * (GLYPH_WIDTH + GLYPH_SPACING)
    }

    pub fn text_height() -> i32 {
        GLYPH_HEIGHT
    }
}