use chrono::Local;
use druid::commands;

use druid::widget::Controller;
use druid::widget::Flex;
use druid::Color;
use druid::Command;
use druid::Event;
use druid::EventCtx;

use druid::Widget;
use druid::WidgetExt;
use druid::WindowDesc;
use druid::{
    im::Vector,
    image::{ImageBuffer, Rgba},
    AppDelegate, Data, DelegateCtx, Env, ImageBuf, Lens,
};
use native_dialog::FileDialog;
use native_dialog::MessageDialog;
use screenshot_lib::*;
use shortcut_lib::*;
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, str::FromStr};
use EditState::*;

#[derive(Clone, Data, PartialEq, Eq)]
pub enum EditState {
    ShortcutEditing(Action),
    PathEditing,
    MouseDetecting,
    ImageResize,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Data)]
pub enum ViewState {
    MainView,
    MenuView,
}

#[derive(Clone, Debug, PartialEq, Eq, Data)]
pub enum ScreenshotMode {
    Fullscreen,
    Cropped(bool),
}

#[derive(Clone, Debug, PartialEq, Eq, Data)]
struct Options {
    save_path: SavePath,
    timer: u64,
    extension: String,
    shortcuts: Shortcuts,
    screen_index: usize,
}

impl Options {
    pub fn new() -> Options {
        Options {
            save_path: SavePath::new(),
            timer: 0,
            extension: String::from_str("jpg").unwrap(),
            shortcuts: Shortcuts::new(),
            screen_index: 0,
        }
    }

    pub fn update_shortcuts(&mut self, action: Action, new_value: (usize, char)) {
        self.shortcuts.update_value(action, new_value);
    }
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    name: String,
    #[data(eq)]
    buf_save: ImageBuffer<Rgba<u8>, Vec<u8>>,
    buf_view: ImageBuf,
    text_buffer: String, //campo da ricontrollare
    view_state: ViewState,
    edit_state: EditState,
    screenshot_mode: (ScreenshotMode, u64),
    options: Options,
    timer: f64,
    area_to_crop: Area,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            name: format!("Screenshot App"),
            buf_save: ImageBuffer::default(),
            buf_view: ImageBuf::empty(),
            text_buffer: String::new(),
            view_state: ViewState::MainView,
            edit_state: EditState::None,
            screenshot_mode: (ScreenshotMode::Fullscreen, u64::default()),
            options: Options::new(),
            timer: 0.0,
            area_to_crop: Area::new(),
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
        return self.options.save_path.get_save_path().clone();
    }

    pub fn get_extension(&self) -> String {
        return self.options.extension.clone();
    }

    pub fn set_extension(&mut self, new_extension: String) {
        self.options.extension = new_extension;
    }

    pub fn get_shortcuts(&self) -> Shortcuts {
        return self.options.shortcuts.clone();
    }

    pub fn get_view_state(&self) -> ViewState {
        return self.view_state.clone();
    }

    pub fn set_view_state(&mut self, value: ViewState) {
        self.edit_state = None;
        self.view_state = value;
    }

    pub fn set_edit_state(&mut self, new_state: EditState) {
        self.edit_state = new_state;
    }

    pub fn get_edit_state(&self) -> EditState {
        self.edit_state.clone()
    }

    pub fn set_screenshot_mode(&mut self, new_screenshot_mode: ScreenshotMode) {
        self.screenshot_mode.0 = new_screenshot_mode;
    }

    pub fn get_screenshot_mode(&self) -> ScreenshotMode {
        self.screenshot_mode.0.clone()
    }

    pub fn set_screenshot_token(&mut self, token: u64) {
        self.screenshot_mode.1 = token;
    }

    pub fn get_screenshot_token(&mut self) -> u64 {
        return self.screenshot_mode.1.clone();
    }

    pub fn get_timer(&self) -> f64 {
        self.timer
    }

    pub fn set_timer(&mut self, timer: f64) {
        self.timer = timer;
    }

    pub fn get_screen_index(&self) -> usize {
        self.options.screen_index
    }

    pub fn set_screen_index(&mut self, screen_index: usize) {
        self.options.screen_index = screen_index;
    }

    pub fn reset_img(&mut self) {
        self.buf_save = ImageBuffer::default();
        self.buf_view = ImageBuf::empty();
    }

    pub fn update_shortcuts(&mut self, action: Action, new_value: (usize, char)) {
        self.options.update_shortcuts(action, new_value);
    }

    fn set_area_to_crop(&mut self, area: Area) {
        self.area_to_crop = area;
    }

    pub fn clear_highlight(&mut self) {
        let (width, height) = (self.buf_save.width(), self.buf_save.height());
        let container = self.get_buf_save().into_raw();
        let new_buf_view = ImageBuf::from_raw(
            container,
            druid::piet::ImageFormat::RgbaSeparate,
            width as usize,
            height as usize,
        );
        self.set_buf((self.get_buf_save(), new_buf_view));
    }

    pub fn highlight_area(&mut self, start_point: (u32, u32), end_point: (u32, u32)) {
        let img_size = (self.buf_save.width(), self.buf_save.height());
        match calculate_area(img_size, start_point, end_point) {
            Option::Some(area) => {
                let (offset_c, offset_r) = area.left_corner;
                //(offset_c, offset_r, width, height)
                let mut container = self.get_buf_save().into_raw();
                for i in (0..container.len()).step_by(4) {
                    container[(i + 3) as usize] = 255;
                }
                for r in 0..area.height {
                    let from = ((r + offset_r) * img_size.0 + offset_c) * 4;
                    let to = ((r + offset_r) * img_size.0 + offset_c + area.width) * 4;
                    for y in (from..to).step_by(4) {
                        container[(y + 3) as usize] = 150;
                    }
                }
                let new_buf_view = ImageBuf::from_raw(
                    container,
                    druid::piet::ImageFormat::RgbaSeparate,
                    img_size.0 as usize,
                    img_size.1 as usize,
                );
                self.set_area_to_crop(area);
                self.set_buf((self.get_buf_save(), new_buf_view));
            }
            Option::None => return,
        }
    }

    pub fn resize_img(&mut self) {
        let img_size = (self.buf_save.width(), self.buf_save.height());
        let (offset_c, offset_r) = self.area_to_crop.left_corner;
        let width = self.area_to_crop.width;
        let height = self.area_to_crop.height;
        let old_container = self.get_buf_save().into_raw();
        let mut new_container: Vec<u8> = vec![];
        for r in 0..height {
            let from = ((r + offset_r) * img_size.0 + offset_c) * 4;
            let to = ((r + offset_r) * img_size.0 + offset_c + width) * 4;
            let mut next_row: Vec<u8> = Vec::from(&old_container[from as usize..to as usize]);
            new_container.append(&mut next_row)
        }
        let new_buf_save =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, old_container)
                .unwrap();
        let new_buf_view = ImageBuf::from_raw(
            new_buf_save.clone().to_vec(),
            druid::piet::ImageFormat::RgbaSeparate,
            width as usize,
            height as usize,
        );

        self.set_buf((new_buf_save, new_buf_view));
    }

    pub fn save_img(&self) {
        let mut path = self.get_save_path();
        let extension = self.get_extension();
        let img = self.get_buf_save();
        if img.is_empty() {
            MessageDialog::new()
                .set_title("Error in saving image")
                .set_text("Do first a screenshot!")
                .set_type(native_dialog::MessageType::Warning)
                .show_alert()
                .unwrap();
            return;
        }
        thread::spawn(move || {
            let default_file_name = format!("image {}", Local::now().format("%y-%m-%d %H%M%S"));
            path.push(default_file_name);
            path.set_extension(extension);
            img.save(path).expect("Error in saving image!");
        });
    }
    pub fn save_img_as(&self) {
        let default_file_name = format!("image {}", Local::now().format("%y-%m-%d %H%M%S")); //name from timestamp
        let path = self.get_save_path();
        let img = self.get_buf_save();
        if img.is_empty() {
            MessageDialog::new()
                .set_title("Error in saving image")
                .set_text("Do first a screenshot!")
                .set_type(native_dialog::MessageType::Warning)
                .show_alert()
                .unwrap();
            return;
        }
        thread::spawn(move || {
            match FileDialog::new()
                .set_filename(&default_file_name)
                .set_location(&path)
                .add_filter("JPG", &["jpg", "jpeg", "jpe", "jfif"])
                .add_filter("PNG", &["png"])
                .add_filter("GIF", &["gif"]) //le gif non vanno
                .show_save_single_file()
                .unwrap()
            {
                Some(path) => img.save(path).expect("Error in saving image!"),
                Option::<PathBuf>::None => {}
            }
        });
    }
}

pub struct EventHandler {
    keys_pressed: Vector<druid::keyboard_types::Key>,
    _valid_shortcut: bool,
    start_point: (u32, u32),
    end_point: (u32, u32),
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: Vector::new(),
            _valid_shortcut: false,
            start_point: (u32::default(), u32::default()),
            end_point: (u32::default(), u32::default()),
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
            druid::Event::Timer(ref timer_event) => {
                if data.get_screenshot_token() == timer_event.into_raw() {
                    match data.get_screenshot_mode() {
                        ScreenshotMode::Fullscreen => {
                            data.set_buf(take_screenshot(data.get_screen_index()).unwrap())
                        }
                        ScreenshotMode::Cropped(ready) => {
                            if ready {
                                data.set_buf(
                                    take_screenshot_area(
                                        data.get_screen_index(),
                                        self.start_point,
                                        self.end_point,
                                    )
                                    .unwrap(),
                                );
                                data.set_edit_state(None);
                                ctx.submit_command(Command::new(
                                    commands::CLOSE_WINDOW,
                                    (),
                                    window_id,
                                ));
                            } else {
                                ctx.new_window(
                                    WindowDesc::new(build_highlighter())
                                        .show_titlebar(false)
                                        .transparent(true)
                                        .set_window_state(druid::WindowState::Maximized)
                                        .set_always_on_top(true),
                                );
                                data.set_edit_state(MouseDetecting);
                            }
                        }
                    }
                }

                data.set_screenshot_token(u64::MAX);

                return Some(event);
            }
            druid::Event::MouseDown(ref mouse_event) => {
                if data.get_edit_state() == MouseDetecting {
                    let start_point: (u32, u32) = (
                        mouse_event.pos.x.ceil() as u32,
                        mouse_event.pos.y.ceil() as u32,
                    );

                    self.start_point = start_point;
                }

                return Some(event);
            }
            druid::Event::MouseUp(ref mouse_event) => {
                if data.get_edit_state() == MouseDetecting {
                    let end_point: (u32, u32) = (
                        mouse_event.pos.x.ceil() as u32,
                        mouse_event.pos.y.ceil() as u32,
                    );

                    self.end_point = end_point;

                    data.set_screenshot_mode(ScreenshotMode::Cropped(true));
                }

                return Some(event);
            }
            druid::Event::KeyDown(ref key_event) => {
                if let EditState::ShortcutEditing(_) = data.get_edit_state() {
                    if self.keys_pressed.contains(&key_event.key) == false {
                        self.keys_pressed.push_back(key_event.key.clone());
                    }
                }
                return Some(event);
            }
            druid::Event::KeyUp(_) => {
                if let EditState::ShortcutEditing(ref _action) = data.get_edit_state() {
                    //data.get_shortcuts().update_value(action, self.keys_pressed);
                    println!(
                        "Update di {:?} con il buffer {:?}",
                        _action, self.keys_pressed
                    );
                    self.keys_pressed.clear();
                    data.set_edit_state(EditState::None);
                }
                return Some(event);
            }
            _ => return Some(event),
        }
    }
}

fn build_highlighter() -> impl Widget<AppState> {
    let timer_sender = TimerSender::new();
    Flex::<AppState>::row()
        .background(Color::rgba(177.0, 171.0, 171.0, 0.389))
        .controller(timer_sender)
}

struct TimerSender;

impl TimerSender {
    fn new() -> TimerSender {
        TimerSender
    }
}

impl<W: Widget<AppState>> Controller<AppState, W> for TimerSender {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        if let Event::MouseUp(_) = event {
            let token = ctx.request_timer(Duration::from_millis(600));
            data.set_screenshot_token(token.into_raw());
            let mut win = ctx.window().clone();
            win.set_window_state(druid::WindowState::Minimized);
        }
        child.event(ctx, event, data, env)
    }
}
