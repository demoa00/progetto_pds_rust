pub mod canvas;

use arboard::{Clipboard, ImageData};
use canvas::canvas::Canvas;
use chrono::Local;
use druid::{
    commands,
    im::{HashMap, Vector},
    image::{ImageBuffer, Rgba},
    keyboard_types::Key,
    piet::ImageFormat,
    widget::{Controller, Flex},
    AppDelegate, Color, Command, Data, DelegateCtx, Env, Event, EventCtx, ImageBuf, Lens, Selector,
    Target, Widget, WidgetExt, WindowDesc,
};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use screenshot_lib::*;
use shortcut_lib::*;
use std::{borrow::Cow, path::PathBuf, str::FromStr, thread, time::Duration};
use EditState::*;

#[derive(Clone, Data, PartialEq, Eq)]
pub enum EditState {
    ShortcutEditing(Action),
    PathEditing,
    MouseDetecting,
    ImageResize,
    Drawing,
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
    extension: String,
    shortcuts: Shortcuts,
}

impl Options {
    pub fn new() -> Options {
        Options {
            save_path: SavePath::new(),
            extension: String::from_str("jpg").unwrap(),
            shortcuts: Shortcuts::new(),
        }
    }

    pub fn update_shortcuts(
        &mut self,
        action: Action,
        new_key_combination: Vector<Key>,
    ) -> Result<(), String> {
        return self.shortcuts.update_value(action, new_key_combination);
    }

    pub fn update_save_path(&mut self) {
        match FileDialog::new().show_open_single_dir() {
            Ok(new_path_opt) => match new_path_opt {
                Some(new_path) => {
                    self.save_path.update_save_path(new_path);
                    return;
                }
                Option::None => {}
            },
            Err(e) => panic!("{e}"),
        }

        MessageDialog::new()
            .set_title("Unable to update save path")
            .set_text("Selected path is not valid!")
            .set_type(MessageType::Warning)
            .show_alert()
            .unwrap();

        return;
    }
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    #[data(ignore)]
    name: String,
    buf_view: ImageBuf,
    text_buffer: String,
    view_state: ViewState,
    edit_state: EditState,
    screenshot_mode: (ScreenshotMode, u64),
    options: Options,
    timer: f64,
    screen_index: usize,
    #[data(ignore)]
    area_to_crop: Area,
    #[data(ignore)]
    pub canvas: Canvas,
    #[data(ignore)]
    thickness: f64,
    #[data(ignore)]
    is_img_empty: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            name: format!("Screenshot App"),
            buf_view: ImageBuf::empty(),
            text_buffer: String::new(),
            view_state: ViewState::MainView,
            edit_state: EditState::None,
            screenshot_mode: (ScreenshotMode::Fullscreen, u64::default()),
            options: Options::new(),
            timer: 0.0,
            screen_index: 0,
            area_to_crop: Area::new(),
            canvas: Canvas::new(),
            thickness: 1.0,
            is_img_empty: true,
        }
    }

    pub fn get_name(&self) -> String {
        return self.clone().name;
    }

    pub fn set_buf(&mut self, buf: ImageBuf) {
        self.is_img_empty = false;

        let is_empty = self.buf_view.raw_pixels().is_empty();

        self.buf_view = buf;

        if is_empty {
            self.copy_to_clipboard();
        }
    }

    pub fn get_buf_view(&self) -> ImageBuf {
        return self.buf_view.clone();
    }

    pub fn copy_to_clipboard(&self) {
        let mut clipboard = Clipboard::new().unwrap();
        let img = ImageData {
            width: self.buf_view.width() as usize,
            height: self.buf_view.height() as usize,
            bytes: Cow::from(self.buf_view.clone().raw_pixels().to_vec()),
        };

        clipboard
            .set_image(img)
            .expect("Unable to copy image on clipboard");
    }

    pub fn get_save_path_for_save(&mut self) -> PathBuf {
        return self.options.save_path.get_save_path_for_save().clone();
    }

    pub fn get_save_path_for_view(&self) -> PathBuf {
        return self.options.save_path.get_save_path_for_view().clone();
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
        if self.edit_state != Drawing {
            self.canvas.set_shape(canvas::canvas::Shape::None)
        }
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

    pub fn set_timer(&mut self, timer: f64) {
        self.timer = timer;
    }

    pub fn get_timer(&self) -> f64 {
        self.timer
    }

    pub fn set_thickness(&mut self, thickness: f64) {
        self.thickness = thickness;
    }

    pub fn get_thickness(&self) -> f64 {
        self.thickness
    }

    pub fn set_screen_index(&mut self, screen_index: usize) {
        self.screen_index = screen_index;
    }

    pub fn get_screen_index(&self) -> usize {
        self.screen_index
    }

    pub fn get_text_buffer(&self) -> String {
        self.text_buffer.clone()
    }

    pub fn get_is_img_empty(&self) -> bool {
        return self.is_img_empty;
    }

    pub fn reset_img(&mut self) {
        self.is_img_empty = true;

        self.buf_view = ImageBuf::empty();
    }

    pub fn update_shortcuts(
        &mut self,
        action: Action,
        new_key_combination: Vector<Key>,
    ) -> Result<(), String> {
        return self.options.update_shortcuts(action, new_key_combination);
    }

    pub fn update_save_path(&mut self) {
        self.options.update_save_path();
    }

    fn set_area_to_crop(&mut self, area: Area) {
        self.area_to_crop = area;
    }

    pub fn clear_highlight(&mut self) {
        let (width, height) = (self.buf_view.width(), self.buf_view.height());
        let mut container = self.buf_view.raw_pixels().to_vec();
        for i in (0..container.len()).step_by(4) {
            container[(i + 3) as usize] = 255;
        }
        let new_buf_view = ImageBuf::from_raw(
            container,
            druid::piet::ImageFormat::RgbaSeparate,
            width as usize,
            height as usize,
        );
        self.set_buf(new_buf_view);
    }

    pub fn highlight_area(&mut self, start_point: (i32, i32), end_point: (i32, i32)) {
        let img_size = (self.buf_view.width() as u32, self.buf_view.height() as u32);

        let mut start_point_cut = (0, 0);
        start_point_cut.0 = if start_point.0 > end_point.0 {
            end_point.0 as usize
        } else {
            start_point.0 as usize
        };
        start_point_cut.1 = if start_point.1 > end_point.1 {
            end_point.1 as usize
        } else {
            start_point.1 as usize
        };

        self.canvas.start_point_cut = start_point_cut;

        match calculate_area(img_size, start_point, end_point) {
            Option::Some(area) => {
                let (offset_c, offset_r) = area.left_corner;
                let mut container = self.buf_view.raw_pixels().to_vec();

                for i in (0..container.len()).step_by(4) {
                    container[(i + 3) as usize] = 255;
                }
                for r in 0..area.height {
                    let from = ((r + offset_r) * img_size.0 + offset_c) * 4;
                    let to = ((r + offset_r) * img_size.0 + offset_c + area.width) * 4;
                    for y in (from..to).step_by(4) {
                        container[(y + 3) as usize] = 100;
                    }
                }
                let new_buf_view = ImageBuf::from_raw(
                    container,
                    druid::piet::ImageFormat::RgbaSeparate,
                    img_size.0 as usize,
                    img_size.1 as usize,
                );
                self.set_area_to_crop(area);
                self.set_buf(new_buf_view);
            }
            Option::None => return,
        }
    }

    pub fn resize_img(&mut self) {
        let img_size = (self.buf_view.width() as u32, self.buf_view.height() as u32);
        let (offset_c, offset_r) = self.area_to_crop.left_corner;
        let width = self.area_to_crop.width;
        let height = self.area_to_crop.height;
        let old_container = self.buf_view.raw_pixels().to_vec();
        let mut new_container: Vec<u8> = vec![];

        for r in 0..height {
            let from = ((r + offset_r) * img_size.0 + offset_c) * 4;
            let to = ((r + offset_r) * img_size.0 + offset_c + width) * 4;
            let mut next_row: Vec<u8> = Vec::from(&old_container[from as usize..to as usize]);
            for c in (0..next_row.len()).step_by(4) {
                next_row[(c + 3) as usize] = 255;
            }
            new_container.append(&mut next_row)
        }

        let new_buf_save =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, new_container)
                .unwrap();
        let new_buf_view = ImageBuf::from_raw(
            new_buf_save.clone().to_vec(),
            ImageFormat::RgbaSeparate,
            width as usize,
            height as usize,
        );

        let old_width = self.buf_view.width();

        self.set_buf(new_buf_view);

        let mut new_modified_pixel = HashMap::new();
        self.canvas.modified_pixel.iter().for_each(|e| {
            let mut new_point = e.0.to_owned();

            new_point.0 = match (new_point.0 / ImageFormat::RgbaSeparate.bytes_per_pixel())
                .checked_sub(self.canvas.start_point_cut.0)
            {
                Some(sub) => sub,
                Option::None => {
                    return;
                }
            };
            new_point.1 = match (new_point.1
                / (old_width * ImageFormat::RgbaSeparate.bytes_per_pixel()))
            .checked_sub(self.canvas.start_point_cut.1)
            {
                Some(sub) => sub,
                Option::None => {
                    return;
                }
            };

            if new_point.0 < self.buf_view.width() && new_point.1 < self.buf_view.height() {
                new_modified_pixel.insert(
                    (
                        new_point.0 * ImageFormat::RgbaSeparate.bytes_per_pixel(),
                        new_point.1
                            * self.buf_view.width()
                            * ImageFormat::RgbaSeparate.bytes_per_pixel(),
                    ),
                    e.1.to_owned(),
                );
            }
        });

        self.canvas.modified_pixel = new_modified_pixel;
    }

    pub fn save_img(&mut self) {
        let mut path = self.get_save_path_for_save();
        let extension = self.get_extension();
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.buf_view.width() as u32,
            self.buf_view.height() as u32,
            self.buf_view.clone().raw_pixels().to_vec(),
        )
        .unwrap();

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

            match img.save(&path) {
                Ok(_) => {}
                Err(_) => {
                    MessageDialog::new()
                        .set_title("Error in saving image")
                        .set_text(&format!("Unable to save image to default path: {}", path.to_str().unwrap()))
                        .set_type(native_dialog::MessageType::Error)
                        .show_alert()
                        .unwrap();
                }
            }
        });
    }

    pub fn save_img_as(&mut self) {
        let default_file_name = format!("image {}", Local::now().format("%y-%m-%d %H%M%S")); //name from timestamp
        let path = self.get_save_path_for_save();
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            self.buf_view.width() as u32,
            self.buf_view.height() as u32,
            self.buf_view.clone().raw_pixels().to_vec(),
        )
        .unwrap();

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
                .add_filter("GIF", &["gif"])
                .show_save_single_file()
                .unwrap()
            {
                Some(path) => match img.save(&path) {
                    Ok(_) => {}
                    Err(_) => {
                        MessageDialog::new()
                            .set_title("Error in saving image")
                            .set_text(&format!("Unable to save image to selected path: {}", path.to_str().unwrap()))
                            .set_type(native_dialog::MessageType::Error)
                            .show_alert()
                            .unwrap();
                    }
                },
                Option::<PathBuf>::None => {}
            }
        });
    }
}

#[derive(Debug)]
pub struct EventHandler {
    keys_pressed: Vector<druid::keyboard_types::Key>,
    start_point: (i32, i32),
    end_point: (i32, i32),
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: Vector::new(),
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
            Event::Timer(ref timer_event) => {
                if data.get_screenshot_token() == timer_event.into_raw() {
                    if data.get_screen_index() > (number_of_screens() - 1) {
                        data.set_screen_index(0);
                    }

                    match data.get_screenshot_mode() {
                        ScreenshotMode::Fullscreen => {
                            data.set_buf(
                                take_screenshot_with_delay(data.timer, data.get_screen_index())
                                    .unwrap(),
                            );
                        }
                        ScreenshotMode::Cropped(ready) => {
                            if ready {
                                data.set_buf(
                                    take_screenshot_area(0, self.start_point, self.end_point)
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
            Event::MouseDown(ref mouse_event) => {
                if data.get_edit_state() == MouseDetecting {
                    let start_point: (i32, i32) = (
                        mouse_event.pos.x.ceil() as i32,
                        mouse_event.pos.y.ceil() as i32,
                    );

                    self.start_point = start_point;
                }
                
                return Some(event);
            }
            Event::MouseUp(ref mouse_event) => {
                if data.get_edit_state() == MouseDetecting {
                    let end_point: (i32, i32) = (
                        mouse_event.pos.x.ceil() as i32,
                        mouse_event.pos.y.ceil() as i32,
                    );

                    self.end_point = end_point;

                    data.set_screenshot_mode(ScreenshotMode::Cropped(true));
                }

                return Some(event);
            }
            Event::KeyDown(ref key_event) => {
                if let EditState::ShortcutEditing(_) = data.get_edit_state() {
                    if self.keys_pressed.contains(&key_event.key) == false
                        && self.keys_pressed.len() < 3
                    {
                        self.keys_pressed.push_back(key_event.key.clone());
                        data.text_buffer = self
                            .keys_pressed
                            .iter()
                            .enumerate()
                            .map(|k| {
                                if k.0 != self.keys_pressed.len() - 1 {
                                    return format!("{} + ", k.1.to_string());
                                } else {
                                    return format!("{}", k.1.to_string());
                                }
                            })
                            .collect();
                    }
                }
                return Some(event);
            }
            Event::KeyUp(_) => {
                if let EditState::ShortcutEditing(ref action) = data.get_edit_state() {
                    match data.update_shortcuts(action.clone(), self.keys_pressed.clone()) {
                        Ok(_) => {}
                        Err(err) => {
                            MessageDialog::new()
                                .set_title("Unable to update shortcut")
                                .set_text(&(err + "\nTry again!"))
                                .set_type(native_dialog::MessageType::Warning)
                                .show_alert()
                                .unwrap();
                        }
                    }

                    self.keys_pressed.clear();
                    data.text_buffer.clear();
                    data.set_edit_state(EditState::None);
                }
                return Some(event);
            }
            Event::WindowSize(_) => {
                ctx.submit_command(Command::new(Selector::new("resize"), (), Target::Auto));
                return Some(event);
            }
            _ => Some(event),
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
    fn new() -> Self {
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
