use chrono::offset::Local;
use directories::UserDirs;
use druid::{
    im::Vector,
    image::{ImageBuffer, Rgba},
    piet::ImageFormat,
    AppDelegate, Data, DelegateCtx, Env, ImageBuf, Lens,
};
use screenshots::Screen;
use shortcut_lib::*;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    str::FromStr,
};

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

#[derive(Clone, Debug, PartialEq, Eq, Data)]
pub enum ViewState {
    MainView,
    MenuView,
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    name: String,
    buf: ImageBuf,
    shortcut: Shortcuts,
    save_path: String,
    view_state: ViewState,
    pub taking_muose_position: bool,
}

impl AppState {
    fn retrive_save_path() -> String {
        let dirs = match UserDirs::new() {
            Some(d) => d,
            None => panic!("Error finding user path!"),
        };

        let new_path = String::from_str(dirs.home_dir().to_str().unwrap()).unwrap();

        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("./path")
        {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        let mut buf: Vec<u8> = Vec::new();

        if file.read_to_end(&mut buf).unwrap() == 0 {
            file.write_all(&bincode::serialize(&new_path).unwrap())
                .expect("File writing failed!");

            return String::from_str(dirs.home_dir().to_str().unwrap()).unwrap();
        } else {
            let path: String = bincode::deserialize::<String>(&buf).unwrap().to_string();

            return path;
        }
    }

    pub fn new() -> Self {
        Self {
            name: format!("Screenshot App"),
            buf: ImageBuf::empty(),
            shortcut: Shortcuts::from_file(),
            save_path: AppState::retrive_save_path(),
            view_state: ViewState::MainView,
            taking_muose_position: false,
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

    pub fn get(&self) -> String {
        self.save_path.clone()
    }

    pub fn set_save_path(&mut self, new_path: String) {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open("./path")
        {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        file.write_all(&bincode::serialize(&new_path).unwrap())
            .expect("File writing failed!");
    }

    pub fn get_save_path(&self) -> String {
        self.save_path.clone()
    }

    pub fn get_view_state(&self) -> ViewState {
        self.view_state.clone()
    }

    pub fn set_view_state(&mut self, value: ViewState) {
        self.view_state = value;
    }

    pub fn is_taking_mouse_position(&self) -> bool {
        self.taking_muose_position
    }

    pub fn set_taking_mouse_position(&mut self, value: bool) {
        self.taking_muose_position = value;
    }

    pub fn get_shortcuts(&self) -> Vec<(String, String)> {
        self.shortcut.to_string()
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
