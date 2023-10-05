use druid::{AppLauncher, LocalizedString, Size, WindowDesc};
use event_lib::*;
use gui_lib::*;
use screenshot_lib::screen_size;

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Screenshot App");
const WINDOW_MIN_SIZE: Size = Size::new(1100.0, 700.0);

fn main() {
    let initial_state = AppState::new();
    let screen_size = screen_size();

    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(|winid, data, _| build_menu(winid, data))
        .window_size(WINDOW_MIN_SIZE)
        .set_position((
            screen_size.width / 2.0 - WINDOW_MIN_SIZE.width / 2.0,
            screen_size.height / 2.0 - WINDOW_MIN_SIZE.height / 2.0,
        ))
        .with_min_size(WINDOW_MIN_SIZE);

    AppLauncher::with_window(main_window)
        .delegate(EventHandler::new())
        .launch(initial_state)
        .expect("Failed to launch application");
}
