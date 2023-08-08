use druid::{
    widget::{Align, Button, Flex, Image, Label, TextBox, ViewSwitcher},
    AppLauncher, Env, LocalizedString, Widget, WidgetExt, WindowDesc,
};
use event_lib::*;

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Screenshot App");

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    let initial_state = AppState::new();

    AppLauncher::with_window(main_window)
        .delegate(EventHandler::new())
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppState> {
    let label = Label::new(|data: &AppState, _env: &Env| format!("Hello {}!", data.get_name()));

    let textbox = TextBox::new()
        .with_placeholder("Who are we greeting?")
        .fix_width(TEXT_BOX_WIDTH)
        .lens(AppState::name);

    let button = Button::new("Click me!").on_click(|_ctx, data: &mut AppState, _env| {
        data.set_buf(event_lib::take_screenshot(0));
    });

    let viewer = ViewSwitcher::new(
        |data: &AppState, _| data.get_buf(),
        |_, data, _| Box::new(Image::new(data.get_buf())),
    );

    let layout = Flex::column()
        .with_child(label)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(textbox)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(button)
        .with_spacer(VERTICAL_WIDGET_SPACING)
        .with_child(viewer);

    Align::centered(layout)
}
