use image::ColorType;
use image::ImageFormat;
use screenshots::Screen;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use std::thread;

//DEMO MAIN : only to try screenshots crate
fn main() {
    let start = Instant::now();
    let screens = Screen::all().unwrap();
    let path1 = Path::new("c:/Users/belal/OneDrive/Desktop/immagine.jpeg");
    let path2 = Path::new("c:/Users/belal/OneDrive/Desktop/immagine.png");
    let path3 = Path::new("c:/Users/belal/OneDrive/Desktop/immagineRidotta.png");

    for screen in screens {
        println!("capturer {screen:?}");
        let image = screen.capture().unwrap();
        let obj = image.rgba();

        image::save_buffer_with_format(
            path1,
            obj,
            image.width(),
            image.height(),
            ColorType::Rgba8,
            ImageFormat::Jpeg,
        )
        .ok();

        image::save_buffer_with_format(
            path2,
            obj,
            image.width(),
            image.height(),
            ColorType::Rgba8,
            ImageFormat::Png,
        )
        .ok();
        /*
        image = screen.capture_area(300, 300, 300, 300).unwrap();
        buffer = image.to_png(None).unwrap();
        fs::write(
            format!(
                "c:/Users/belal/OneDrive/Desktop/{}-2.png",
                screen.display_info.id
            ),
            buffer,
        )
        .unwrap();
        */
    }

    let screen = Screen::from_point(100, 100).unwrap();
    println!("capturer {screen:?}");

    let image = screen.capture_area(300, 300, 300, 300).unwrap();
    let obj2 = image.rgba();
    image::save_buffer_with_format(
        path3,
        obj2,
        image.width(),
        image.height(),
        ColorType::Rgba8,
        ImageFormat::Png,
    )
    .ok();

    println!("运行耗时: {:?}", start.elapsed());
}

/*

  This function recieve a String containing the path,
  then it saves a screenshot of the whole selected screen

*/
pub fn take_screenshot(save_path: String) {
    let screens = Screen::all().unwrap();
    let path = Path::new(save_path.as_str());
    let format =
        take_format(save_path.clone()).expect("Error! Format not supported for this release!");
    for screen in screens {
        let image = screen.capture().unwrap();
        //"screenshot" is the buffer of u8 (=>&Vec<u8>), ready to be saved
        let screenshot = image.rgba();
        image::save_buffer_with_format(
            path,
            screenshot,
            image.width(),
            image.height(),
            ColorType::Rgba8,
            format,
        )
        .ok();
    }

    return;
}

/*

  This function recieve a String containing the path and a delay expressed in u64,
  then it calls "take_screenshot"

*/

pub fn take_screenshot_with_delay(save_path: String, time: u64) {
    let sleep_time = Duration::new(time, 0.0 as u32);
    thread::sleep(sleep_time);
    take_screenshot(save_path);
    return;
}

pub fn take_format(save_path: String) -> Option<ImageFormat> {
    if save_path.contains(".jpeg") {
        return Some(ImageFormat::Jpeg);
    } else if save_path.contains(".png") {
        return Some(ImageFormat::Png);
    } else if save_path.contains(".avif") {
        return Some(ImageFormat::Avif);
    } else if save_path.contains(".bmp") {
        return Some(ImageFormat::Bmp);
    } else if save_path.contains(".dds") {
        return Some(ImageFormat::Dds);
    } else if save_path.contains(".gif") {
        return Some(ImageFormat::Gif);
    } else if save_path.contains(".hdr") {
        return Some(ImageFormat::Hdr);
    } else if save_path.contains(".ico") {
        return Some(ImageFormat::Ico);
    } else if save_path.contains(".farbfeld") {
        return Some(ImageFormat::Farbfeld);
    } else if save_path.contains(".openexr") {
        return Some(ImageFormat::OpenExr);
    } else if save_path.contains(".pnm") {
        return Some(ImageFormat::Pnm);
    } else if save_path.contains(".qoi") {
        return Some(ImageFormat::Qoi);
    } else if save_path.contains(".tga") {
        return Some(ImageFormat::Tga);
    } else if save_path.contains(".tiff") {
        return Some(ImageFormat::Tiff);
    } else if save_path.contains(".webp") {
        return Some(ImageFormat::WebP);
    } else {
        return None;
    }
}
