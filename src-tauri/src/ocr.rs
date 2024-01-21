use std::error::Error;
use active_win_pos_rs::get_position;
use rusty_tesseract::{Args, Image, image_to_string};
use rusty_tesseract::image::DynamicImage;
use screenshots::Screen;

/// Wow this isn't working well at all!
pub fn get_text_on_screen() -> Result<String, Box<dyn Error>> {

    let screens = Screen::all()?;
    let screen = screens.first().unwrap();
    let buffer = screen.capture()?;
    let mut image = DynamicImage::ImageRgba8(buffer.clone());
    let s = screen.display_info.scale_factor as f64;

    if let Ok(pos) = get_position() {
        image.crop((pos.x * s) as u32, (pos.y * s) as u32, (pos.width * s) as u32, (pos.height * s) as u32);
        println!("{}, {}, {}, {}", pos.x, pos.y, pos.width, pos.height);
    } else {
        println!("Failed to crop image");
    }

    perform_ocr(&image)
}

fn perform_ocr(dynamic_image: &DynamicImage) -> Result<String, Box<dyn std::error::Error>> {
    let args = Args::default();
    let image = Image::from_dynamic_image(dynamic_image).unwrap();
    Ok(image_to_string(&image, &args)?)
}