use image::*;
use screenshots::{DisplayInfo, Screen};
use std::path::Path;
use std::thread;
use std::time::Duration;

/*

  This function recieve the current screen on witch the screenshot has to be taken,
  then it saves a screenshot of the whole selected screen in a ImageBuffer

*/

pub fn take_screenshot(current_screen: usize) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let screens = Screen::all().unwrap();
    let current_screen = screens[current_screen];
    let image = current_screen.capture().unwrap();
    return image;
}

/*

  This function recieve the current screen on witch the screenshot has to be taken,
  In this case, the area is also passed after a drag&drop in order to take a restricted area
  then it saves a screenshot of the whole selected area in a ImageBuffer
  It returns an Option so, in case the selected area is bigger than the current selected screen, None is returned 

*/

pub fn take_screenshot_area(current_screen: usize, start_coords: (i32, i32), end_coords: (i32, i32)) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let screens = Screen::all().unwrap();
    let current_screen = screens[current_screen];
    let screen_infos = current_screen.display_info;
    let infos = calculate_area(screen_infos, start_coords, end_coords);
    match infos {
        Some(info) => {
            let image = current_screen
                .capture_area(info.0, info.1, info.2 as u32, info.3 as u32)
                .unwrap();
            return Some(image);
        }
        _ => return None,
    }
}

/*

  This function verifies if the drag&drop comes from left to right, form top to bottom or viceversa, then i calculates
  the top left corner and verifies if the dimensions of the area are compatibles with the current screen 

*/

pub fn calculate_area( screen_infos: DisplayInfo, start_coords: (i32, i32), end_coords: (i32, i32)) -> Option<(i32, i32, i32, i32)> {
    let mut left_corner = (0, 0);
    let x_diff = start_coords.0 - end_coords.0;
    let y_diff = start_coords.1 - end_coords.1;
    let width;
    let height;
    // from right to left
    if x_diff > 0 {
        left_corner.0 = end_coords.0;
        width = x_diff;
    } else {
        left_corner.0 = start_coords.0;
        width =- x_diff;
    }
    // from bottom to top
    if y_diff > 0 {
        left_corner.1 = end_coords.1;
        height = y_diff;
    } else {
        left_corner.1 = start_coords.1;
        height =- y_diff;
    }
    // if the top left corner + the 2 dimension are bigger than the screen sizes the screenshot is NOT valid
    if (width + left_corner.0) > screen_infos.width as i32 || (height + left_corner.1) > screen_infos.height as i32
    {
        return None;
    }
    return Some((left_corner.0, left_corner.1, width, height));
}

/*

  This function recieves an ImageBuffer and a path (with the extension) and saves it

*/

pub fn save_screenshot(save_path: String, screenshot: ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let path = Path::new(save_path.as_str());
    let format =
        take_format(save_path.clone()).expect("Error! Format not supported for this release!");
    image::save_buffer_with_format(
        path,
        screenshot.as_bytes(),
        screenshot.width(),
        screenshot.height(),
        ColorType::Rgba8,
        format,
    )
    .ok();
    return;
}

/*

  This function recieve a delay expressed in u64 and the current screen,
  then it calls "take_screenshot"

*/

pub fn take_screenshot_with_delay(time: u64, current_screen: usize) {
    let sleep_time = Duration::new(time, 0.0 as u32);
    thread::sleep(sleep_time);
    take_screenshot(current_screen);
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
