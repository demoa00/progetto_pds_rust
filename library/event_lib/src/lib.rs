use druid::commands;
use druid::Command;
use druid::{
    im::Vector,
    image::{ImageBuffer, Rgba},
    AppDelegate, Data, DelegateCtx, Env, ImageBuf, Lens,
};
use screenshot_lib::*;
use shortcut_lib::*;
use std::{path::PathBuf, str::FromStr};
use EditState::*;

#[derive(Clone, Data, PartialEq, Eq)]
pub enum EditState {
    ShortcutEditing((String, String)),
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
    buf_save: ImageBuffer<Rgba<u8>, Vec<u8>>,
    buf_view: ImageBuf,
    #[data(ignore)]
    save_path: PathBuf,
    default_extension: String,
    default_shortcut: Shortcuts,
    text_buffer: String, //campo da ricontrollare
    view_state: ViewState,
    edit_state: EditState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            name: format!("Screenshot App"),
            buf_save: ImageBuffer::default(),
            buf_view: ImageBuf::empty(),
            save_path: SavePath::new().get_save_path(),
            default_extension: String::from_str("jpg").unwrap(),
            default_shortcut: Shortcuts::new(),
            text_buffer: String::new(),
            view_state: ViewState::MainView,
            edit_state: EditState::None,
        }
    }

    pub fn get_name(&self) -> String {
        return self.clone().name;
    }

    pub fn set_buf(&mut self, buf: (ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuf)) {
        self.buf_save = buf.0;
        self.buf_view = buf.1;
    }

    pub fn get_buf_save(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        return self.buf_save.clone();
    }

    pub fn get_buf_view(&self) -> ImageBuf {
        return self.buf_view.clone();
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

    pub fn get_default_shortcut(&self) -> Shortcuts {
        return self.default_shortcut.clone();
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
    start_point: (i32, i32),
    end_point: (i32, i32),
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            _keys_pressed: Vector::new(),
            start_point: (i32::default(), i32::default()),
            end_point: (i32::default(), i32::default()),
        }
    }
}

impl AppDelegate<AppState> for EventHandler {
    fn event(
        &mut self,
        ctx: &mut DelegateCtx,
        window_id: druid::WindowId,
        event: druid::Event,
        data: &mut AppState,
        _env: &Env,
    ) -> Option<druid::Event> {
        match event {
            druid::Event::Timer(ref _timer_event) => {
                //data.set_buf(take_screenshot(0));
                data.set_edit_state(MouseDetecting);

                data.get_default_shortcut().reset();

                return Some(event);
            }
            druid::Event::MouseDown(ref mouse_event) => {
                if data.get_edit_state() == MouseDetecting {
                    let start_point: (i32, i32) = (
                        mouse_event.pos.x.ceil() as i32,
                        mouse_event.pos.y.ceil() as i32,
                    );

                    self.start_point = start_point;

                    println!("{:?}", self.start_point);
                }

                return Some(event);
            }
            druid::Event::MouseUp(ref mouse_event) => {
                if data.get_edit_state() == MouseDetecting {
                    let end_point: (i32, i32) = (
                        mouse_event.pos.x.ceil() as i32,
                        mouse_event.pos.y.ceil() as i32,
                    );

                    self.end_point = end_point;

                    println!("{:?}", self.end_point);

                    data.set_buf(
                        take_screenshot_area(0, self.start_point, self.end_point).unwrap(),
                    ); //da cambiare

                    data.set_edit_state(None);

                    ctx.submit_command(Command::new(commands::CLOSE_WINDOW, (), window_id));
                }

                return Some(event);
            }
            _ => return Some(event),
        }
    }
}
