use druid::{widget::*, Color};
use druid::{ImageBuf, Widget, WidgetExt};
use event_lib::*;
mod transparent_button;
use transparent_button::transparent_button::*;

const UI_IMG_PATH: &str = "../library/gui_lib/ui_img";
const TOP_BAR_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::TEAL);
const BOTTOM_PAGE_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::WHITE);

pub fn build_root_widget() -> impl Widget<AppState> {
    let top_bar = ViewSwitcher::new(
        |data: &AppState, _| data.get_view_state(),
        move |selector, _, _| Box::new(build_top_bar_widget(selector)),
    );

    let bottom_page = ViewSwitcher::new(
        |data: &AppState, _| data.get_view_state(),
        move |selector, _, _| Box::new(build_bottom_page_widget(selector)),
    );

    Flex::column()
        .with_flex_child(top_bar.background(TOP_BAR_COLOR), 1.0)
        .with_default_spacer()
        .with_flex_child(bottom_page.background(BOTTOM_PAGE_COLOR), 1.0)
        .background(BOTTOM_PAGE_COLOR)
}

fn build_top_bar_widget(view_state: &ViewState) -> impl Widget<AppState> {
    match view_state {
        ViewState::MainView => {
            let button_new_screenshot = TransparentButton::with_bg(
                Image::new(ImageBuf::from_file(format!("{}/new.png", UI_IMG_PATH)).unwrap()),
                |_, data: &mut AppState, _| {
                    /*ctx.submit_command(druid::commands::HIDE_WINDOW);
                    data.set_taking_mouse_position(true);*/
                    data.set_buf(take_screenshot(0));
                },
            );
            let button_options = TransparentButton::with_bg(
                Image::new(ImageBuf::from_file(format!("{}/options.png", UI_IMG_PATH)).unwrap()),
                |_, data: &mut AppState, _| data.set_view_state(ViewState::MenuView),
            );
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
            let button_return = TransparentButton::with_bg(
                Image::new(ImageBuf::from_file(format!("{}/return.png", UI_IMG_PATH)).unwrap()),
                |_, data: &mut AppState, _| data.set_view_state(ViewState::MainView),
            );
            Flex::row()
                .main_axis_alignment(MainAxisAlignment::End)
                .must_fill_main_axis(true)
                .with_flex_child(button_return, 1.0)
        }
    }
}

fn build_bottom_page_widget(view_state: &ViewState) -> impl Widget<AppState> {
    match view_state {
        ViewState::MainView => {
            let screeshot_viewer = ViewSwitcher::new(
                |data: &AppState, _| data.get_buf(),
                |_, data, _| Box::new(Image::new(data.get_buf())),
            );
            Flex::column()
                .with_child(screeshot_viewer)
                .padding((20.0, 20.0))
                .center()
        }

        ViewState::MenuView => Flex::column()
            .with_child(Label::new("Work in progress").with_text_color(Color::rgb(0.0, 0.0, 0.0)))
            .center(),
    }
}
