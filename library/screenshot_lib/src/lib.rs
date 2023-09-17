use druid::{Data, ImageBuf};
use image::*;
use screenshots::Screen;
use std::{thread, time::Duration};


#[derive(Clone, Debug, PartialEq, Eq, Data)]
pub struct Area {
    pub left_corner: (u32, u32),
    pub width: u32,
    pub height: u32,
}

impl Area {
    pub fn new() -> Area {
        Area {
            left_corner: (u32::default(), u32::default()),
            width: u32::default(),
            height: u32::default(),
        }
    }
}

/// This function recieve the current screen on witch the screenshot has to be taken,
/// then it saves a screenshot of the whole selected screen in a ImageBuffer.
pub fn take_screenshot(
    current_screen: usize,
) -> Option<(ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf)> {
    let screens = Screen::all().unwrap();
    let current_screen = screens[current_screen];
    let screen_infos = current_screen.display_info;

    let image = current_screen.capture().unwrap();

    let img_vec = image.clone().to_vec();

    let image_view = ImageBuf::from_raw(
        img_vec,
        druid::piet::ImageFormat::RgbaSeparate,
        screen_infos.width as usize,
        screen_infos.height as usize,
    );
    return Some((image, image_view));
}

/// This function recieve the current screen on witch the screenshot has to be taken.
/// In this case, the area is also passed after a drag&drop in order to take a restricted area
/// then it saves a screenshot of the whole selected area in a ImageBuffer.
/// It returns an Option so, in case the selected area is bigger than the current selected screen, None is returned.
pub fn take_screenshot_area(
    current_screen: usize,
    start_coords: (u32, u32),
    end_coords: (u32, u32),
) -> Option<(ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf)> {
    let screens = Screen::all().unwrap();
    let current_screen = screens[current_screen];
    let screen_infos = current_screen.display_info;
    let calculated_area = calculate_area(
        (screen_infos.width, screen_infos.height),
        start_coords,
        end_coords,
    );

    match calculated_area {
        Some(area) => {
            let image = current_screen
                .capture_area(
                    area.left_corner.0 as i32,
                    area.left_corner.1 as i32,
                    area.width,
                    area.height,
                )
                .unwrap();
            let img_vec = image.clone().to_vec();

            let image_view = ImageBuf::from_raw(
                img_vec,
                druid::piet::ImageFormat::RgbaSeparate,
                area.width as usize,
                area.height as usize,
            );
            return Some((image, image_view));
        }
        _ => return None,
    }
}

/// This function verifies if the drag&drop comes from left to right, form top to bottom or viceversa, then i calculates
/// the top left corner and verifies if the dimensions of the area are compatibles with the current screen.
pub fn calculate_area(
    (screen_width, screen_height): (u32, u32),
    mut start_coords: (u32, u32),
    mut end_coords: (u32, u32),
) -> Option<Area> {
    // the screenshot area is between the current screen and a screen on his left
    /* if start_coords.0 < 0 {
        start_coords.0 = 0;
    }
    if end_coords.0 < 0 {
        end_coords.0 = 0;
    } */
    // the screenshot area is between the current screen and a screen on his right
    if start_coords.0 > screen_width {
        start_coords.0 = screen_width;
    }
    if end_coords.0 > screen_width {
        end_coords.0 = screen_width;
    }
    // the screenshot area is between the current screen and a screen on his top
    /* if start_coords.1 < 0 {
        start_coords.1 = 0;
    }
    if end_coords.1 < 0 {
        end_coords.1 = 0;
    } */
    // the screenshot area is between the current screen and a screen on his right
    if start_coords.1 > screen_height {
        start_coords.1 = screen_height;
    }
    if end_coords.1 > screen_height {
        end_coords.1 = screen_height;
    }

    let mut left_corner: (u32, u32) = (0, 0);
    let x_diff: i32 = start_coords.0 as i32 - end_coords.0 as i32;
    let y_diff: i32 = start_coords.1 as i32 - end_coords.1 as i32;
    let width: u32;
    let height: u32;

    // from right to left
    if x_diff > 0 {
        left_corner.0 = end_coords.0;
    } else {
        left_corner.0 = start_coords.0;
    }
    width = x_diff.abs() as u32;

    // from bottom to top
    if y_diff > 0 {
        left_corner.1 = end_coords.1;
    } else {
        left_corner.1 = start_coords.1;
    }
    height = y_diff.abs() as u32;

    // if the top left corner + the 2 dimension are bigger than the screen sizes the screenshot is NOT valid
    if (width + left_corner.0) > screen_width || (height + left_corner.1) > screen_height {
        return None;
    }
    return Some(Area {
        left_corner,
        width,
        height,
    });
}

/// This function recieve a delay expressed in u64 and,
/// the current screen then it calls `take_screenshot`.
pub fn take_screenshot_with_delay(
    time: u64,
    current_screen: usize,
) -> Option<(ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf)> {
    let sleep_time = Duration::new(time, 0.0 as u32);

    thread::sleep(sleep_time);

    return take_screenshot(current_screen);
}
