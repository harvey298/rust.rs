use std::path::PathBuf;
use image::{DynamicImage, ImageBuffer, Rgba};

use anyhow::{Result, Ok};
use std::fs;


pub fn get_servers(data_dir: PathBuf) -> Result<Vec<String>> {
    let mut servers = Vec::new();
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let filename = file_name.to_string_lossy().to_owned();
        servers.push(filename.to_string())
    }

    Ok(servers)
}

pub fn text_to_image(text: &str) -> DynamicImage {
    // Load the font from a file
    let font = include_bytes!("./assets/OpenSans-Italic-VariableFont_wdth,wght.ttf");
    let font = rusttype::Font::try_from_vec(font.to_vec()).unwrap();

    // Calculate the size of the text
    let scale = rusttype::Scale::uniform(32.0);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, rusttype::point(0.0, 0.0 + v_metrics.ascent)).collect();
    let glyphs_height = v_metrics.ascent - v_metrics.descent;

    // Calculate the bounding box of the text
    let mut x_min = std::f32::INFINITY;
    let mut y_min = std::f32::INFINITY;
    let mut x_max = std::f32::NEG_INFINITY;
    let mut y_max = std::f32::NEG_INFINITY;

    for glyph in &glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            x_min = x_min.min(bb.min.x as f32);
            y_min = y_min.min(bb.min.y as f32);
            x_max = x_max.max(bb.max.x as f32);
            y_max = y_max.max(bb.max.y as f32);
        }
    }

    let text_width = x_max.ceil() as u32 - x_min.floor() as u32;
    let text_height = y_max.ceil() as u32 - y_min.floor() as u32;

    // Create an image buffer for the text
    let mut text_image = ImageBuffer::from_pixel(text_width, text_height, Rgba([0, 0, 0, 0]));

    // Draw the glyphs onto the image buffer
    for glyph in &glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let px = (x as i32 + bb.min.x) as u32;
                let py = (y as i32 + bb.min.y) as u32;
                let color = (v * 255.0) as u8;
                let px = px.saturating_sub(x_min as u32).clamp(0, text_width - 1);
                let py = py.saturating_sub(y_min as u32).clamp(0, text_height - 1);
                text_image.put_pixel(px, py, Rgba([0, 0, 0, color]));
            });
        }
    }

    DynamicImage::ImageRgba8(text_image)
}
