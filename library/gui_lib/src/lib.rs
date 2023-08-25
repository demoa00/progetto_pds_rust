use druid::{widget::*, Color};
use druid::{ImageBuf, Widget, WidgetExt};
use event_lib::*;
use shortcut_lib::*;
mod transparent_button;
use transparent_button::transparent_button::*;
mod modified_flex;
use modified_flex::modified_flex::*;
use strum::IntoEnumIterator;

const UI_IMG_PATH: &str = "../library/gui_lib/ui_img";
const TOP_BAR_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::TEAL);
const BOTTOM_PAGE_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::WHITE);

pub fn build_root_widget() -> impl Widget<AppState> {
    let main_view = View::new(ViewState::MainView);
    let menu_view = View::new(ViewState::MenuView);

    Flex::column()
        .with_child(main_view.top_bar)
        .with_child(main_view.bottom_page)
        .with_child(menu_view.top_bar)
        .with_child(menu_view.bottom_page)
        .background(BOTTOM_PAGE_COLOR)
}

pub struct View {
    top_bar: Box<dyn Widget<AppState>>,
    bottom_page: Box<dyn Widget<AppState>>,
}

impl View {
    fn new(view_state: ViewState) -> View {
        View {
            top_bar: Box::new(View::build_top_bar_widget(&view_state)),
            bottom_page: Box::new(View::build_bottom_page_widget(&view_state)),
        }
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
                    Image::new(
                        ImageBuf::from_file(format!("{}/options.png", UI_IMG_PATH)).unwrap(),
                    ),
                    |_, data: &mut AppState, _| data.set_view_state(ViewState::MenuView),
                );
                let left_part = Flex::row()
                    .main_axis_alignment(druid::widget::MainAxisAlignment::Start)
                    .with_flex_child(button_new_screenshot, 1.0)
                    .must_fill_main_axis(false);
                let right_part = Flex::row()
                    .main_axis_alignment(druid::widget::MainAxisAlignment::End)
                    .with_flex_child(button_options, 1.0);
                let split = Split::columns(left_part, right_part).bar_size(0.0);
                FlexMod::column(true)
                    .with_child(split)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MainView)
            }
            ViewState::MenuView => {
                let button_return = TransparentButton::with_bg(
                    Image::new(ImageBuf::from_file(format!("{}/return.png", UI_IMG_PATH)).unwrap()),
                    |_, data: &mut AppState, _| data.set_view_state(ViewState::MainView),
                );
                FlexMod::row(false)
                    .main_axis_alignment(modified_flex::modified_flex::MainAxisAlignment::End)
                    .must_fill_main_axis(true)
                    .with_flex_child(button_return, 1.0)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MenuView)
                    .with_default_spacer()
            }
        }
        .background(TOP_BAR_COLOR)
    }

    fn build_bottom_page_widget(view_state: &ViewState) -> impl Widget<AppState> {
        match view_state {
            ViewState::MainView => {
                let screeshot_viewer = Padding::new(
                    (30.0, 30.0),
                    ViewSwitcher::new(
                        |data: &AppState, _| data.get_buf(),
                        |_, data, _| Box::new(Image::new(data.get_buf())),
                    ),
                );
                FlexMod::column(true)
                    .with_child(screeshot_viewer)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MainView)
                    .center()
                    .background(BOTTOM_PAGE_COLOR)
            }

            ViewState::MenuView => {
                let mut shortcut_menu = MenuOption::new("Shortcut".to_string());
                for action in Action::iter() {
                    shortcut_menu.add_option(
                        action.to_string(),
                        Flex::row()
                            .with_child(Label::new(|_data: &AppState, _: &_| "Alt + F4".to_string() /*get shortcut from action*/).with_text_color(Color::GRAY))
                            .with_child(TransparentButton::with_bg(Image::new(ImageBuf::from_file(format!("{}/edit.png", UI_IMG_PATH)).unwrap()), move |_, _data: &mut AppState, _| println!("Voglio modificare {:?}", action))))
                }
                let mut path_menu = MenuOption::new("Saving".to_string());
                path_menu.add_option(
                    "Path".to_string(),
                    Flex::row()
                        .with_child(
                            Label::new(|data: &AppState, _: &_| data.get_save_path())
                                .with_text_color(Color::GRAY),
                        )
                        .with_child(TransparentButton::with_bg(
                            Image::new(
                                ImageBuf::from_file(format!("{}/edit.png", UI_IMG_PATH)).unwrap(),
                            ),
                            move |_, _data: &mut AppState, _| println!("Voglio modificare il path"),
                        )),
                );
                let menu_options = Flex::column()
                    .with_child(shortcut_menu.build())
                    .with_child(path_menu.build());

                FlexMod::column(false)
                    .with_child(menu_options)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MenuView)
                    .center()
                    .background(BOTTOM_PAGE_COLOR)
            }
        }
    }
}

struct MenuOption {
    title: String,
    options: Vec<Box<dyn Widget<AppState>>>,
}

impl MenuOption {
    fn new(title: String) -> MenuOption {
        MenuOption {
            title: title,
            options: vec![],
        }
    }

    fn add_option(
        self: &mut Self,
        title: String,
        interactive_part: impl Widget<AppState> + 'static,
    ) {
        let option = Split::columns(
            Label::new(title)
                .with_text_color(Color::GRAY)
                .padding((0.0, 15.0)),
            interactive_part.align_right(),
        )
        .bar_size(0.0);
        self.options.push(Box::new(option));
    }

    fn build(self: Self) -> impl Widget<AppState> {
        let mut result = Flex::column().with_child(
            Label::new(self.title)
                .with_text_size(30.0)
                .with_text_color(Color::BLACK)
                .align_left()
                .padding((30.0, 15.0)),
        );
        let mut options = Flex::column();
        for option in self.options {
            options.add_child(option.padding((0.0, 0.0)));
        }
        result.add_child(options.padding((110.0, 0.0)));
        result
    }
}
