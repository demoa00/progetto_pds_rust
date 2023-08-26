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
use EditState::*;

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

#[derive(Clone, Data, PartialEq, Eq)]
pub enum EditState {
    ShortcutEditing(ShortcutKey),
    PathEditing,
    MouseDetecting,
    None,
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
    buffer_path: String,
    view_state: ViewState,
    edit_state: EditState,
}

impl AppState {
    fn retrive_save_path() -> String {
        let dirs = match UserDirs::new() {
            Some(d) => d,
            Option::None => panic!("Error finding user path!"),
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
            buffer_path: String::new(),
            view_state: ViewState::MainView,
            edit_state: EditState::None,
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

    pub fn get_save_path(&self) -> String {
        self.save_path.clone()
    }

    pub fn set_save_path(&mut self) {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open("./path")
        {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        file.write_all(&bincode::serialize(&self.buffer_path).unwrap())
            .expect("File writing failed!");

        self.save_path = self.buffer_path.clone();
    }

    pub fn get_view_state(&self) -> ViewState {
        self.view_state.clone()
    }

    pub fn set_view_state(&mut self, value: ViewState) {
        // Al cambio di view si interrompe la ricezione di eventi
        self.edit_state = None;
        self.view_state = value;
    }

    pub fn get_shortcuts(&self) -> Vec<(String, String)> {
        self.shortcut.to_string()
    }

    pub fn get_edit_state(&self) -> EditState {
        self.edit_state.clone()
    }

    pub fn set_edit_state(&mut self, value: EditState) {
        match &value {
            PathEditing => self.buffer_path = self.save_path.clone(),
            _ => {}
        }
        self.edit_state = value;
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
                    //Shortcuts::new_shortcut_to_file(&self.keys_pressed);
                    match &data.edit_state {
                        ShortcutEditing(old_shortcut) => {
                            data.shortcut
                                .edit_shortcut(old_shortcut, &self.keys_pressed);

                            data.edit_state = EditState::None;
                        }
                        _ => {}
                    }

                    match data.shortcut.extract_value(&self.keys_pressed) {
                        Some(action) => match action {
                            Action::NewScreenshot => {
                                take_screenshot(0);
                            }
                            Action::Save => {
                                function_2(3);
                            }
                        },
                        Option::None => {}
                    }
                }

                return Some(event);
            }

            druid::Event::KeyUp(ref key_event) => {
                let index = match self.keys_pressed.index_of(&key_event.key) {
                    Some(i) => i,
                    Option::None => panic!("Key searched doesn't exist!"),
                };

                self.keys_pressed.remove(index);

                println!("Buffer {:?}", self.keys_pressed);
                return Some(event);
            }

            _ => Some(event),
        }
    }

    /*fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppState,
        env: &Env,
    ) -> druid::Handled {
        println!("Oscuro");
        if cmd.is(HIDE_WINDOW) {
            println!("Oscuro");
            // Start timer and at the end chiamare ctx.submit_command(druid::commands::SHOW_WINDOW);
        }
        if cmd.is(SHOW_WINDOW) {
            // Take screenshot
        }

        druid::Handled::Yes
    }*/
}

#[cfg(test)]
mod tests {
    /* use super::*;

    #[test]
    fn it_works() {
        /* Write here */
    } */
}
