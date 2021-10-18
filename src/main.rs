#![windows_subsystem = "windows"]

use adhaan::*;
use druid::{AppLauncher, Data, WindowDesc};

mod config;
mod ui;
mod utils;
#[allow(unused)]
mod widgets;

use config::*;

#[derive(Clone, Data)]
pub struct AppState {
    pub config: Config,

    #[data(same_fn = "PartialEq::eq")]
    pub prayers: PrayerTimes,
}

#[cfg(windows)]
pub fn main() -> anyhow::Result<()> {
    let main_window = WindowDesc::new(ui::main_root())
        .title("Adhaan")
        .show_titlebar(false)
        .show_in_taskbar(false)
        .set_always_on_top(true)
        .resizable(false)
        .window_size(ui::size::MAIN_WINDOW)
        .set_level(druid::WindowLevel::AppWindow);

    let config = Config::load().unwrap();
    let initial_state = AppState {
        prayers: PrayerTimes::calculate(
            chrono::Local::today().naive_local(),
            config.coordinates,
            config.method.get_parameters(),
        )
        .unwrap(),
        config,
    };

    let (tray_event_tx, tay_event_rx) = std::sync::mpsc::channel::<()>();

    let _tray_icon = trayicon::TrayIconBuilder::new()
        .tooltip("Adhaan")
        .icon_from_buffer(ui::TRAY_ICON)
        .on_click(())
        .sender(tray_event_tx)
        .build()
        .map_err(|e| anyhow::format_err!("Tray error: {:?}", e))?;

    let app_launcher = AppLauncher::with_window(main_window)
        .configure_env(move |env, data| {
            Theme::load(&data.config.theme)
                .unwrap_or_default()
                .apply_to_env(env);
        })
        .delegate(ui::AppDelegate);

    let ext_events_tray = app_launcher.get_external_handle();
    std::thread::spawn(move || {
        for _ in tay_event_rx {
            ext_events_tray
                .submit_command(ui::selector::SHOW, (), druid::Target::Auto)
                .unwrap();
        }
    });

    app_launcher.log_to_console().launch(initial_state)?;

    Ok(())
}

#[cfg(not(windows))]
pub fn main() {
    eprintln!("Only available for windows now.")
}
