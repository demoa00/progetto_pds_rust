use druid::widget::*;
use druid::{ImageBuf, Widget, WidgetExt};
use event_lib::*;
mod transparent_button;
use transparent_button::transparent_button::*;

const BUTTON_DIM: (f64, f64) = (40.0, 40.0);
const UI_IMG_PATH: &str = "../library/gui_lib/ui_img";

enum ViewState {
    MainView,
    MenuView,
}

pub fn build_root_widget() -> impl Widget<AppState> {
    let top_bar = ViewSwitcher::new(
        |data: &AppState, _| data.get_main_ui(),
        move |_, data, _| {
            if data.get_main_ui() {
                Box::new(build_top_bar_widget(ViewState::MainView))
            } else {
                Box::new(build_top_bar_widget(ViewState::MenuView))
            }
        },
    );

    let bottom_page = build_bottom_page();

    let result = Flex::column()
        .with_flex_child(top_bar, 1.0)
        .with_default_spacer()
        .with_flex_child(bottom_page, 1.0);
    return result;
}

fn build_top_bar_widget(view_state: ViewState) -> impl Widget<AppState> {
    let img_new = Image::new(ImageBuf::from_file(format!("{}/new.png", UI_IMG_PATH)).unwrap());
    let img_options =
        Image::new(ImageBuf::from_file(format!("{}/options.png", UI_IMG_PATH)).unwrap());
    let img_return =
        Image::new(ImageBuf::from_file(format!("{}/return.png", UI_IMG_PATH)).unwrap());

    return match view_state {
        ViewState::MainView => {
            let button_new_screenshot = ZStack::new(img_new)
                .with_centered_child(
                    TransparentButton::new("")
                        .on_click(|_, data: &mut AppState, _| data.set_buf(take_screenshot(0)))
                        .fix_size(BUTTON_DIM.0, BUTTON_DIM.1),
                )
                .fix_size(BUTTON_DIM.0, BUTTON_DIM.1);
            let button_options = ZStack::new(img_options)
                .with_centered_child(
                    TransparentButton::new("")
                        .on_click(|_, data: &mut AppState, _| data.set_main_ui(false))
                        .fix_size(BUTTON_DIM.0, BUTTON_DIM.1),
                )
                .fix_size(BUTTON_DIM.0, BUTTON_DIM.1);
            let left_part = Flex::row()
                .main_axis_alignment(MainAxisAlignment::Start)
                .with_flex_child(button_new_screenshot, 1.0)
                .must_fill_main_axis(false);
            let right_part = Flex::row()
                .main_axis_alignment(MainAxisAlignment::End)
                .with_flex_child(button_options, 1.0);
            let split = Split::columns(left_part, right_part).bar_size(0.0);
            Flex::column().with_child(split)
        }
        ViewState::MenuView => {
            let button_return = ZStack::new(img_return)
                .with_centered_child(
                    TransparentButton::new("")
                        .on_click(|_, data: &mut AppState, _| data.set_main_ui(true))
                        .fix_size(BUTTON_DIM.0, BUTTON_DIM.1),
                )
                .fix_size(BUTTON_DIM.0, BUTTON_DIM.1);
            Flex::row()
                .main_axis_alignment(MainAxisAlignment::End)
                .must_fill_main_axis(true)
                .with_flex_child(button_return, 1.0)
        }
    };
}

fn build_bottom_page() -> impl Widget<AppState> {
    // Main view bottom page
    let screenshot_viewer = ViewSwitcher::new(
        |data: &AppState, _| data.get_buf(),
        |_, data, _| Box::new(Image::new(data.get_buf())),
    );
    let main_bottom_page =
        DisabledIf::new(screenshot_viewer, |data: &AppState, _| !data.get_main_ui());

    // Main view bottom page
    let menu_botttom_page =
        DisabledIf::new(Label::new("Work in progress"), |data: &AppState, _| {
            data.get_main_ui()
        });

    return ZStack::new(main_bottom_page).with_centered_child(menu_botttom_page);
}
