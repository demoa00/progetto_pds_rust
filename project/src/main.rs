use druid::{AppLauncher, LocalizedString, WindowDesc};
use event_lib::*;
use gui_lib::*;

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Screenshot App");

fn main() {
    let initial_state = AppState::new();

    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(|winid, data, _| build_menu(winid, data))
        .window_size((1100.0, 650.0))
        .with_min_size((1100.0, 650.0));

    AppLauncher::with_window(main_window)
        .delegate(EventHandler::new())
        .launch(initial_state)
        .expect("Failed to launch application");
}
