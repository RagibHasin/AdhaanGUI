#![windows_subsystem = "windows"]

use adhaan::*;

mod app_state;
mod config;
mod ui_about;
mod ui_main;
mod ui_settings;
mod utils;
#[allow(unused)]
mod widgets;

use crate::{app_state::*, config::*};

#[cfg(windows)]
pub fn main() -> anyhow::Result<()> {
    let config = Config::load().unwrap();

    let main_window = druid::WindowDesc::new(ui_main::main_root())
        .title("Adhaan")
        .show_titlebar(false)
        .show_in_taskbar(false)
        .set_always_on_top(true)
        .resizable(false)
        .window_size(ui_main::size::MAIN_WINDOW)
        .set_level(druid::WindowLevel::AppWindow);

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
        .icon_from_buffer(ui_main::TRAY_ICON)
        .on_click(())
        .sender(tray_event_tx)
        .build()
        .map_err(|e| anyhow::format_err!("Tray error: {:?}", e))?;

    let app_launcher = druid::AppLauncher::with_window(main_window)
        .configure_env(|env, app_state: &AppState| {
            app_state.config.apply_appearance_to_env(env);
        })
        .delegate(ui_main::AppDelegate(None));

    let ext_events_tray = app_launcher.get_external_handle();
    std::thread::spawn(move || {
        for _ in tay_event_rx {
            ext_events_tray
                .submit_command(ui_main::selector::SHOW, (), druid::Target::Auto)
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
