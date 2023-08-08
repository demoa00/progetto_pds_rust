use chrono::offset::Local;
use druid::{
    im::Vector,
    image::{ImageBuffer, Rgba},
    piet::ImageFormat,
    AppDelegate, Data, DelegateCtx, Env, ImageBuf, Lens,
};
use screenshots::Screen;
use shortcut_lib::*;

/// Constant value of heigth of screen
const H: usize = 1080;
/// Constant value of width of screen
const W: usize = 1920;

/// Function associated to shortcut enum
/// ___Action::NewScreenshot___. When it is execute
/// produce in output a screenshot of the entire
/// desired screen
pub fn take_screenshot(screen_index: usize) -> ImageBuf {
    let img = Screen::all().unwrap()[screen_index].capture().unwrap();
    let x: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(W as u32, H as u32, img.rgba().clone()).unwrap();

    let dt = Local::now();
    x.save(dt.timestamp().to_string() + ".jpg")
        .expect("Error in saving screenshot!");

    let raw = Screen::all().unwrap()[screen_index]
        .capture()
        .unwrap()
        .rgba()
        .clone();

    return ImageBuf::from_raw(
        &raw[0..H * W * ImageFormat::RgbaSeparate.bytes_per_pixel()],
        ImageFormat::RgbaSeparate,
        W,
        H,
    );
}

fn function_2(num: usize) -> bool {
    println!("Save {num}");
    return true;
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    name: String,
    buf: ImageBuf,
    shortcut: Shortcuts,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            name: format!("Screenshot App"),
            buf: ImageBuf::empty(),
            shortcut: Shortcuts::from_file(),
        }
    }

    pub fn get_name(&self) -> String {
        self.clone().name
    }

    pub fn set_buf(&mut self, buf: ImageBuf) {
        self.buf = buf;
    }

    pub fn get_buf(&self) -> ImageBuf {
        self.clone().buf
    }
}

pub struct EventHandler {
    keys_pressed: Vector<druid::keyboard_types::Key>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: Vector::new(),
        }
    }
}

impl AppDelegate<AppState> for EventHandler {
    fn event(
        &mut self,
        _ctx: &mut DelegateCtx,
        _window_id: druid::WindowId,
        event: druid::Event,
        data: &mut AppState,
        _env: &Env,
    ) -> Option<druid::Event> {
        match event {
            druid::Event::KeyDown(ref key_event) => {
                if self.keys_pressed.contains(&key_event.key.clone()) == false {
                    self.keys_pressed.push_back(key_event.key.clone());
                }

                println!("Buffer {:?}", self.keys_pressed);

                if self.keys_pressed.len() == 2 {
                    match data.shortcut.extract_value(&self.keys_pressed) {
                        Some(action) => match action {
                            Action::NewScreenshot => {
                                take_screenshot(0);
                            }
                            Action::Save => {
                                function_2(3);
                            }
                        },
                        None => {}
                    }
                }

                return Some(event);
            }

            druid::Event::KeyUp(ref key_event) => {
                let index = match self.keys_pressed.index_of(&key_event.key) {
                    Some(i) => i,
                    None => panic!("Key searched doesn't exist!"),
                };

                self.keys_pressed.remove(index);

                println!("Buffer {:?}", self.keys_pressed);
                return Some(event);
            }

            _ => Some(event),
        }
    }
}

#[cfg(test)]
mod tests {
    /* use super::*;

    #[test]
    fn it_works() {
        /* Write here */
    } */
}
