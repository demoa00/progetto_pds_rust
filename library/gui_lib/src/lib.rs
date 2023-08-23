use druid::{widget::*, Color};
use druid::{ImageBuf, Widget, WidgetExt};
use event_lib::*;
mod transparent_button;
use transparent_button::transparent_button::*;
mod modified_flex;
use modified_flex::modified_flex::*;

const UI_IMG_PATH: &str = "../library/gui_lib/ui_img";
const TOP_BAR_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::TEAL);
const BOTTOM_PAGE_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::WHITE);

/*pub struct View {
    top_bar: Rc<Container<AppState>>,
    bottom_page: Rc<Container<AppState>>,
    state: ViewState,
}

impl View {
    fn new(view_state: ViewState) -> View {
        View {
            top_bar: Rc::new(build_top_bar_widget(&view_state)),
            bottom_page: Rc::new(build_bottom_page_widget(&view_state)),
            state: view_state,
        }
    }


}*/

pub fn build_root_widget() -> impl Widget<AppState> {
    let main_top_bar = build_top_bar_widget(ViewState::MainView);
    let menu_top_bar = build_top_bar_widget(ViewState::MenuView);
    let main_bottom_page = build_bottom_page_widget(ViewState::MainView);
    let menu_bottom_page = build_bottom_page_widget(ViewState::MenuView);

    Flex::column()
        .with_child(main_top_bar)
        .with_child(main_bottom_page)
        .with_child(menu_top_bar)
        .with_child(menu_bottom_page)
        .background(BOTTOM_PAGE_COLOR)

    /*druid::widget::Flex::column()
        .with_child(
            FlexMod::column()
                .with_child(build_top_bar_widget(&ViewState::MainView))
                .visible_if(|data| data.get_view_state() == ViewState::MainView),
        )
        .with_child(Label::new("GG"))

    let mut cache: HashMap<ViewState, Container<AppState>> = HashMap::new();

    cache.insert(
        ViewState::MainView,
        Flex::column()
            .with_flex_child(build_top_bar_widget(&ViewState::MainView), 1.0)
            .with_default_spacer()
            .with_flex_child(build_bottom_page_widget(&ViewState::MainView), 1.0)
            .background(BOTTOM_PAGE_COLOR),
    );

    cache.insert(
        ViewState::MenuView,
        Flex::column()
            .with_flex_child(build_top_bar_widget(&ViewState::MenuView), 1.0)
            .with_default_spacer()
            .with_flex_child(build_bottom_page_widget(&ViewState::MenuView), 1.0)
            .background(BOTTOM_PAGE_COLOR),
    );

    let t = build_top_bar_widget(&ViewState::MainView);
    View

    let main_view = View::new(ViewState::MainView);
    let menu_view = View::new(ViewState::MenuView);

    ViewSwitcher::new(
        |data: &AppState, _| data.get_view_state(),
        move |selector, _, _| {
            let x = t.clone();

            Box::new(x)
        },
    )

    top_bars.insert(
        ViewState::MainView,
        build_top_bar_widget(&ViewState::MainView),
    );

    let top_bar = ViewSwitcher::new(
        |data: &AppState, _| data.get_view_state(),
        move |selector, _, _| Box::new(),
    );

    let bottom_page = ViewSwitcher::new(
        |data: &AppState, _| data.get_view_state(),
        move |selector, _, _| Box::new(build_bottom_page_widget(selector)),
    );

    Flex::column()
        .with_flex_child(top_bar.background(TOP_BAR_COLOR), 1.0)
        .with_default_spacer()
        .with_flex_child(bottom_page.background(BOTTOM_PAGE_COLOR), 1.0)
        .background(BOTTOM_PAGE_COLOR)*/
}

fn build_top_bar_widget(view_state: ViewState) -> impl Widget<AppState> {
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

fn build_bottom_page_widget(view_state: ViewState) -> impl Widget<AppState> {
    match view_state {
        ViewState::MainView => {
            let screeshot_viewer = ViewSwitcher::new(
                |data: &AppState, _| data.get_buf(),
                |_, data, _| Box::new(Image::new(data.get_buf())),
            );
            FlexMod::column(true)
                .with_child(screeshot_viewer)
                .visible_if(|data: &AppState| data.get_view_state() == ViewState::MainView)
                //.padding((20.0, 20.0))
                .center()
                .background(BOTTOM_PAGE_COLOR)
        }

        ViewState::MenuView => FlexMod::column(false)
            .with_child(Label::new("Work in progress").with_text_color(Color::rgb(0.0, 0.0, 0.0)))
            .visible_if(|data: &AppState| data.get_view_state() == ViewState::MenuView)
            .center()
            .background(BOTTOM_PAGE_COLOR),
    }
}
