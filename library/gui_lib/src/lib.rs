mod button_mod;
mod flex_mod;
pub mod canvas_widget;
use button_mod::druid_mod::*;
use canvas_widget::canvas_widget::CanvasWidget;
use druid::{
    widget::{*, self}, Color, Env, ImageBuf, KeyOrValue, LocalizedString, Menu, MenuItem, Widget,
    WidgetExt, WindowId, Command, Selector, Target, Event,
};
use event_lib::*;
use flex_mod::druid_mod::*;
use shortcut_lib::*;
use screenshot_lib::number_of_screens;
use core::panic;
use std::time::Duration;
use strum::IntoEnumIterator;
use native_dialog::MessageDialog;

const UI_IMG_PATH: &str = "../library/gui_lib/ui_img";
const TOP_BAR_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::TEAL);
const BOTTOM_PAGE_COLOR: BackgroundBrush<AppState> = BackgroundBrush::Color(Color::WHITE);

pub fn build_menu(_window: Option<WindowId>, _data: &AppState) -> Menu<event_lib::AppState> {
    let mut base = Menu::empty();

    base = base.entry(
        Menu::new(LocalizedString::new("common-menu-file-menu"))
            .entry(
                MenuItem::new("New screenshot")
                    .on_activate(move |ctx, _data: &mut AppState, _| {
                        ctx.submit_command(Command::new(Selector::new("new_screenshot"), (), Target::Auto));
                    })
                    .dynamic_hotkey(|data: &AppState, _env: &Env| {
                        data.get_shortcuts()
                            .extract_value_for_menu(Action::NewScreenshot)
                    }),
            )
            .entry(
                MenuItem::new("Save")
                    .on_activate(|_ctx, data: &mut AppState, _| data.save_img())
                    .dynamic_hotkey(|data: &AppState, _env: &Env| {
                        data.get_shortcuts().extract_value_for_menu(Action::Save)
                    }),
            )
            .entry(
                MenuItem::new("Save as...")
                    .on_activate(|_ctx, data: &mut AppState, _| data.save_img_as())
                    .dynamic_hotkey(|data: &AppState, _env: &Env| {
                        data.get_shortcuts().extract_value_for_menu(Action::SaveAs)
                    }),
            )
            .enabled_if(|data: &AppState, _| {
                if (data.get_edit_state() == EditState::None) || (data.get_edit_state() == EditState::Drawing) {
                    if data.get_view_state() == ViewState::MainView {
                        true
                    }else{
                        false
                    }
                }else{
                    false
                }
            }),
    );

    return base;
}

pub fn build_root_widget() -> impl Widget<AppState> {
    let main_view = View::new(ViewState::MainView);
    let menu_view = View::new(ViewState::MenuView);
    let close_controller = WindowController::new();

    Flex::column()
        .with_child(main_view.top_bar)
        .with_child(main_view.bottom_page)
        .with_child(menu_view.top_bar)
        .with_child(menu_view.bottom_page)
        .background(BOTTOM_PAGE_COLOR)
        .controller(close_controller)
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
                let normal_top_bar = {
                    let button_new_screenshot_full = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/fullscreen.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |ctx, data: &mut AppState, _| {
                            data.set_edit_state(EditState::None);
                            prepare_for_screenshot(data, ctx, ScreenshotMode::Fullscreen)
                        },
                    );
                    let button_new_screenshot_cropped = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/crop.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |ctx, data: &mut AppState, _| {
                            data.set_edit_state(EditState::None);
                            prepare_for_screenshot(data, ctx, ScreenshotMode::Cropped(false))
                        },
                    );

                    let button_drawing = FlexMod::row(false).with_flex_child(TransparentButton::with_bg(
                        Image::new(
                        ImageBuf::from_file(format!("{}/edit.png", UI_IMG_PATH)).unwrap(),
                        ), 
                        |_, data: &mut AppState, _| data.set_edit_state(EditState::Drawing),
                    ), 1.0)
                    .visible_if(|data: &AppState| {
                        if !data.get_is_img_empty(){
                            match data.get_edit_state() {
                                EditState::Drawing => false,
                                _ => true,
                            }
                        }else{
                            false
                        }
                    });

                    let button_copy = FlexMod::row(false).with_flex_child(TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/copy.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| data.copy_to_clipboard(),
                    ), 1.0)
                    .visible_if(|data: &AppState| {
                        if !data.get_is_img_empty(){
                            true
                        }else{
                            false
                        }
                    });

                    let button_save = FlexMod::row(false).with_flex_child(TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/save.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| data.save_img(),
                    ), 1.0)
                    .visible_if(|data: &AppState| {
                        if !data.get_is_img_empty(){
                            true
                        }else{
                            false
                        }
                    });
                    
                    let button_options = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/options.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |ctx, data: &mut AppState, _| {
                            if !data.get_is_img_empty() {
                                match MessageDialog::new().set_title("Do you want to exit the editing window?")
                                                            .set_text("If you confirm all changes made and the image will be deleted")
                                                            .show_confirm() {
                                    Ok(confirm) => {
                                        if confirm {
                                            data.reset_img();
                                            data.set_edit_state(EditState::None);
                                            ctx.submit_command(Command::new(Selector::new("update_widget"), (), Target::Auto));
                                            data.set_view_state(ViewState::MenuView);
                                        }
                                    },
                                    Err(e) => panic!("{}", e),
                                }   
                            }else{
                                data.set_view_state(ViewState::MenuView);
                            }
                        },
                    );

                    let left_part = Flex::row()
                        .main_axis_alignment(druid::widget::MainAxisAlignment::Start)
                        .with_flex_child(button_new_screenshot_full, 1.0)
                        .with_flex_child(button_new_screenshot_cropped, 1.0)
                        .must_fill_main_axis(false);

                    let right_part = Flex::row()
                        .main_axis_alignment(druid::widget::MainAxisAlignment::End) 
                        .with_flex_child(button_drawing, 1.0)
                        .with_flex_child(button_copy, 1.0)
                        .with_flex_child(button_save, 1.0)
                        .with_flex_child(button_options, 1.0);

                    FlexMod::row(true).with_flex_child(Split::columns(left_part, right_part).bar_size(0.0).split_point(0.8), 1.0)
                                .visible_if(|data|{
                                    match data.get_edit_state() {
                                        EditState::ImageResize => false,
                                        EditState::MouseDetecting => false,
                                        _ => true,
                                    }
                                })
                };

                let drawing_top_bar = {
                    let button_red_color = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/paint_red.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| { data.canvas.set_color(0xff0000ff);}
                    );
                    let button_green_color = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/paint_green.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_color(0x00ff00ff);}
                    );
                    let button_blue_color = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/paint_blu.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_color(0x0000ffff);}
                    );

                    let button_none = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/return.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.set_edit_state(EditState::None)},
                    );

                    let button_rubber = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/rubber.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_shape(canvas::canvas::Shape::Rubber)},
                    );
                    let button_fill = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/fill.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_fill(!data.canvas.get_fill())}
                    );
                    let button_free = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/free.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_shape(canvas::canvas::Shape::Free)},
                    );
                    let button_line = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/line.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_shape(canvas::canvas::Shape::Line)},
                    );
                    let button_rectangle = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/rectangle.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_shape(canvas::canvas::Shape::Rectangle)},
                    );
                    let button_circle = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/circle.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_shape(canvas::canvas::Shape::Cirle)},
                    );
                    let button_scissors = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/scissors.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {data.canvas.set_shape(canvas::canvas::Shape::Cut)},
                    );

                    FlexMod::row(false)
                    .with_child(Flex::row().with_child(button_red_color).with_child(button_green_color).with_child(button_blue_color).padding((20.0,0.0)))
                    .with_child(Flex::row().with_child(View::build_thickness_slider()).padding((20.0,0.0)))
                    .with_child(Flex::row().with_child(button_free).with_child(button_line).with_child(button_rectangle).with_child(button_circle).with_child(button_rubber).padding((20.0,0.0)))
                    .with_child(Flex::row().with_child(button_fill).padding((20.0,0.0)))
                    .with_child(Flex::row().with_child(button_scissors).padding((20.0,0.0)))
                    .with_child(Flex::row().with_child(button_none).padding((20.0,0.0)))
                    .visible_if(|data: &AppState| data.get_edit_state() == EditState::Drawing).center().border(Color::BLACK, 2.0)
                };

                let resize_top_bar = {
                    let confirm_button = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/check.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {
                            data.resize_img();
                            data.set_edit_state(EditState::Drawing);
                        },
                    );

                    let undo_button = TransparentButton::with_bg(
                        Image::new(
                            ImageBuf::from_file(format!("{}/return.png", UI_IMG_PATH)).unwrap(),
                        ),
                        |_, data: &mut AppState, _| {
                            data.clear_highlight();
                            data.set_edit_state(EditState::Drawing)
                        },
                    );

                    FlexMod::row(false)
                        .with_child(confirm_button)
                        .with_child(undo_button)
                        .visible_if(|data: &AppState| {
                            data.get_edit_state() == EditState::ImageResize
                        })
                        .center()
                };

                FlexMod::column(true)
                    .with_child(normal_top_bar)
                    .with_child(resize_top_bar)
                    .with_child(drawing_top_bar)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MainView)
            }
            ViewState::MenuView => {
                let button_return = TransparentButton::with_bg(
                    Image::new(ImageBuf::from_file(format!("{}/return.png", UI_IMG_PATH)).unwrap()),
                    |_, data: &mut AppState, _| data.set_view_state(ViewState::MainView),
                );

                FlexMod::row(false)
                    .main_axis_alignment(flex_mod::druid_mod::MainAxisAlignment::End)
                    .must_fill_main_axis(true)
                    .with_flex_child(button_return, 1.0)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MenuView)
                    .with_default_spacer()
            }
        }
        .background(TOP_BAR_COLOR)
    }

    fn build_thickness_slider() -> impl Widget<AppState> {
        let thickness_slider = 
            Slider::new()
                .with_range(1.0, 15.0)
                .track_color(KeyOrValue::Concrete(Color::WHITE))
                .knob_style(KnobStyle::Wedge)
                .axis(druid::widget::Axis::Horizontal)
                .with_step(2.0)
                .annotated(2.0, 1.0)
                .fix_width(120.0)
                .lens(AppState::thickness);
        
        return thickness_slider;
    }

    fn build_bottom_page_widget(view_state: &ViewState) -> impl Widget<AppState> {
        match view_state {
            ViewState::MainView => {
                let screeshot_viewer = Padding::new(
                    (30.0, 30.0),
                    ViewSwitcher::new(
                        |data: &AppState, _| data.get_buf_view(),
                        |_, data, _| {
                            return Box::new(Flex::column().with_child(CanvasWidget::new(data.get_buf_view()).controller(WindowController::new())).main_axis_alignment(widget::MainAxisAlignment::Start));
                        },
                    ),
                );

                FlexMod::column(true)
                    .with_child(screeshot_viewer)
                    .visible_if(|data: &AppState| data.get_view_state() == ViewState::MainView)
                    .center()
                    .background(BOTTOM_PAGE_COLOR)
            }
            ViewState::MenuView => {
                let shortcut_menu = MenuOption::build_shortcut_menu_widget();
                let path_menu = MenuOption::build_path_menu_widget();
                let timer_menu = MenuOption::build_timer_menu();
                let screen_menu = MenuOption::build_screen_menu();
                let menu_options = Scroll::new(
                    Flex::column()
                        .with_child(shortcut_menu)
                        .with_child(path_menu)
                        .with_child(timer_menu)
                        .with_child(screen_menu),
                )
                .vertical()
                .fix_height(500.0);

                FlexMod::column(false)
                    .with_flex_child(menu_options, 1.0)
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
        .bar_size(0.0)
        .split_point(0.4);

        self.options.push(Box::new(option));
    }

    fn build(self: Self) -> impl Widget<AppState> {
        let mut result = Flex::column().with_child(
            Label::new(self.title)
                .with_text_size(30.0)
                .with_text_color(Color::BLACK)
                .align_left()
                .padding((40.0, 15.0)),
        );
        let mut options = Flex::column();

        for option in self.options {
            options.add_child(option.padding((0.0, 0.0)));
        }
        result.add_child(options.padding((120.0, 0.0)));
        result
    }

    fn build_path_menu_widget() -> impl Widget<AppState> {
        let mut path_menu = MenuOption::new("Saving".to_string());

        path_menu.add_option(
            "Path".to_string(),
            Flex::row()
                .with_child(
                    Flex::column().with_child(
                        Label::new(|data: &AppState, _: &_| {
                            data.get_save_path_for_view().to_str().unwrap().to_string()
                        })
                        .with_text_color(Color::GRAY),
                    ),
                )
                .with_child(TransparentButton::with_bg(
                    Image::new(ImageBuf::from_file(format!("{}/edit.png", UI_IMG_PATH)).unwrap()),
                    move |_ctx, data: &mut AppState, _| {
                        data.set_edit_state(EditState::PathEditing);
                        data.update_save_path();
                        data.set_edit_state(EditState::None);
                    },
                )),
        );
        path_menu.build()
    }

    fn build_shortcut_menu_widget() -> impl Widget<AppState> {
        let mut shortcut_menu = MenuOption::new("Shortcut".to_string());

        for action in Action::iter() {
            let action_clone = action.clone();
            shortcut_menu.add_option(
                action.to_string(),
                ViewSwitcher::new(
                    move |data: &AppState, _| data.get_edit_state(),
                    move |selector, _, _| {
                        if let EditState::ShortcutEditing(ref action_to_edit) = selector {
                            if &action == action_to_edit {
                                Box::new(
                                    TextBox::new()
                                        .with_placeholder("Press keys")
                                        .lens(AppState::text_buffer)
                                        .padding((0.0, 15.0))
                                        .disabled_if(|_, _| true),
                                )
                            } else {
                                Box::new(Flex::row())
                            }
                        } else {
                            let act = action_clone.clone();
                            let act2 = action_clone.clone();
                            Box::new(
                                Flex::row()
                                    .with_child(ViewSwitcher::new(
                                        move |data: &AppState, _| {
                                            data.get_shortcuts()
                                                .extract_value_for_gui(&act)
                                                .unwrap()
                                        },
                                        |selector, _, _| {
                                            Box::new(
                                                Label::new(selector.to_string())
                                                    .with_text_color(Color::GRAY),
                                            )
                                        },
                                    ))
                                    .with_child(TransparentButton::with_bg(
                                        Image::new(
                                            ImageBuf::from_file(format!(
                                                "{}/edit.png",
                                                UI_IMG_PATH
                                            ))
                                            .unwrap(),
                                        ),
                                        move |_, data: &mut AppState, _| {
                                            data.set_edit_state(EditState::ShortcutEditing(
                                                act2.clone(),
                                            ));
                                        },
                                    )),
                            )
                        }
                    },
                ),
            )
        }
        shortcut_menu.build()
    }

    fn build_timer_menu() -> impl Widget<AppState> {
        let mut timer_menu = MenuOption::new("Timer".to_string());
        timer_menu.add_option(
            "Duration".to_string(),
            Slider::new()
                .with_range(0.0, 10.0)
                .track_color(KeyOrValue::Concrete(Color::TEAL))
                .knob_style(KnobStyle::Wedge)
                .axis(druid::widget::Axis::Horizontal)
                .with_step(1.0)
                .annotated(2.0, 1.0)
                .fix_width(250.0)
                .padding((0.0, 15.0))
                .lens(AppState::timer),
        );
        timer_menu.build()
    }

    fn build_screen_menu() -> impl Widget<AppState> {
        let mut screen_menu = MenuOption::new("Screen".to_string());
        let mut screen_indexes = vec![];
        for i in 0..number_of_screens() {
            screen_indexes.push(((i+1).to_string(), i));
        }
        screen_menu.add_option("Index".to_string(), RadioGroup::row(screen_indexes).lens(AppState::screen_index));
        screen_menu.build().controller(WindowController::new())
    }
}

fn prepare_for_screenshot(data: &mut AppState, ctx: &mut druid::EventCtx, mode: ScreenshotMode) {
    let mut win = ctx.window().clone();
    
    if win.get_window_state() != druid::WindowState::Minimized{
        win.set_window_state(druid::WindowState::Minimized);    
    }
    
    data.reset_img();
    data.set_screenshot_mode(mode);

    let token = ctx.request_timer(Duration::from_millis(data.get_timer() as u64 + 500));
    data.set_screenshot_token(token.into_raw());
}

struct WindowController;

impl WindowController {
    fn new() -> Self{
        WindowController
    }
}

impl<W: Widget<AppState>> Controller<AppState, W> for WindowController {
    fn event(&mut self, child: &mut W, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut AppState, env: &Env) {
        match event {
            Event::Command(ref c) => {
                if c.is(Selector::<()>::new("new_screenshot")){
                    prepare_for_screenshot(data, ctx, ScreenshotMode::Fullscreen);
                }

                child.event(ctx, event, data, env);
            },
            _ => { child.event(ctx, event, data, env);},
        }
    }
}
