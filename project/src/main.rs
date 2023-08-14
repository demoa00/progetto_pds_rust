use druid::{AppLauncher, LocalizedString, WindowDesc};
use event_lib::*;
use gui_lib::*;

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
