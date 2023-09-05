use directories::UserDirs;
use druid::{
    im::Vector,
    image::{ImageBuffer, Rgba},
    piet::ImageFormat,
    AppDelegate, Data, DelegateCtx, Env, ImageBuf, Lens,
};
use screenshots::Screen;
use shortcut_lib::*;
use std::{path::PathBuf, str::FromStr};
use EditState::*;

/// Function associated to shortcut enum
/// ___Action::NewScreenshot___. When it is execute
/// produce in output a screenshot of the entire
/// desired screen
/* pub fn take_screenshot(screen_index: usize) -> ImageBuf {
    let img = Screen::all().unwrap()[screen_index].capture().unwrap();
    let x: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(W as u32, H as u32, img.rgba().clone()).unwrap();

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
} */

pub fn take_screenshot(screen_index: usize) -> (ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf) {
    let screens = Screen::all().unwrap();
    let screen_info = screens[screen_index].display_info;
    let img = screens[screen_index].capture().unwrap();
    let img_vec = img.clone().into_vec();

    let img_buf = ImageBuf::from_raw(
        img_vec,
        ImageFormat::RgbaSeparate,
        screen_info.width as usize,
        screen_info.height as usize,
    );

    return (img, img_buf);
}

/* fn function_2(num: usize) -> bool {
    println!("Save {num}");
    return true;
} */

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
    #[data(ignore)]
    buf: (ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf),
    #[data(ignore)]
    save_path: PathBuf,
    default_extension: String,
    text_buffer: String, //campo da ricontrollare
    view_state: ViewState,
    edit_state: EditState,
}

impl AppState {
    pub fn new() -> Self {
        let user_dirs = match UserDirs::new() {
            Some(d) => d,
            Option::None => panic!("Error finding user's dir path!"),
        };

        let img_dir = match user_dirs.picture_dir() {
            Some(dir) => dir.to_owned(),
            Option::None => panic!("Error finding image dir path!"),
        };

        Self {
            name: format!("Screenshot App"),
            buf: (ImageBuffer::default(), ImageBuf::empty()),
            save_path: img_dir,
            default_extension: String::from_str("jpg").unwrap(),
            text_buffer: String::new(),
            view_state: ViewState::MainView,
            edit_state: EditState::None,
        }
    }

    pub fn get_name(&self) -> String {
        return self.clone().name;
    }

    pub fn set_buf(&mut self, buf: (ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf)) {
        self.buf = buf;
    }

    pub fn get_buf(&self) -> (ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf) {
        return self.buf.clone();
    }

    pub fn get_save_path(&self) -> PathBuf {
        return self.save_path.clone();
    }

    pub fn get_default_extension(&self) -> String {
        return self.default_extension.clone();
    }

    pub fn set_default_extension(&mut self, new_extension: String) {
        self.default_extension = new_extension;
    }

    pub fn get_view_state(&self) -> ViewState {
        return self.view_state.clone();
    }

    pub fn set_view_state(&mut self, value: ViewState) {
        // Al cambio di view si interrompe la ricezione di eventi
        self.edit_state = None;
        self.view_state = value;
    }

    pub fn set_edit_state(&mut self, new_state: EditState) {
        self.edit_state = new_state;
    }

    pub fn get_edit_state(&self) -> EditState {
        self.edit_state.clone()
    }
}

pub struct EventHandler {
    _keys_pressed: Vector<druid::keyboard_types::Key>,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            _keys_pressed: Vector::new(),
        }
    }
}

impl AppDelegate<AppState> for EventHandler {
    fn event(
        &mut self,
        _ctx: &mut DelegateCtx,
        _window_id: druid::WindowId,
        event: druid::Event,
        _data: &mut AppState,
        _env: &Env,
    ) -> Option<druid::Event> {
        match event {
            /* druid::Event::KeyDown(ref key_event) => {
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
                                //take_screenshot(0);
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
            } */
            _ => Some(event),
        }
    }
}
