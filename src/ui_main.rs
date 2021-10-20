use chrono::prelude::*;
use druid::{
    lens, theme,
    widget::{self, prelude::*, CrossAxisAlignment, Flex, FlexParams, Label},
    Command, FontWeight, HasRawWindowHandle, LocalizedString, RawWindowHandle, Scalable, Target,
    WidgetExt, WindowDesc, WindowId, WindowLevel,
};
use extract::*;

use winapi::um::winuser;

use crate::{
    utils::{lens_map_get, DataWrapper},
    widgets::button::Button,
    *,
};

pub mod color {
    use druid::Color;

    druid_widget_nursery::keys! {
        ELAPSED_CRITICAL: Color,
        ELAPSED_OKAY: Color,
        REMAINING: Color,
        CLOSE_HOT: Color,
        CLOSE_ACTIVE: Color,
    }
}

pub mod size {
    use druid::Size;

    pub const MAIN_WINDOW: Size = Size::new(320.0, 400.0);
    pub const CORNER_BUFFER: Size = Size::new(16.0, 16.0);
    pub const ACTIVE_CORNER_RADIUS: f64 = 8.0;
}

pub mod selector {
    druid_widget_nursery::selectors! {
        INIT,
        SHOW,
    }
}

pub static TRAY_ICON: &[u8] = include_bytes!("../resources/icon.ico");

pub fn main_root() -> impl Widget<AppState> {
    let location_name = widget::Maybe::or_empty(|| {
        Flex::row()
            .with_child(
                utils::Icon::Location
                    .label(FontWeight::THIN)
                    .with_text_color(theme::FOREGROUND_DARK),
            )
            .with_child(
                Label::new(|location: &String, _: &Env| location.clone()).with_text_size(24.0),
            )
    })
    .lens(lens!(AppState, config.location_name));

    let title = Flex::row()
        .with_child(
            Label::new(LocalizedString::new("Adhaan").with_placeholder("Adhaan"))
                .with_text_size(24.0),
        )
        .with_child(location_name);

    let asr_row = widget::Either::<AsrLensed>::new(
        |(DataWrapper((_, asr_2)), _): &AsrLensed, _: &Env| asr_2.is_some(),
        Flex::<AsrLensed>::column()
            .with_child(prayer_row().lens(lens_map_get(
                |&(DataWrapper((asr_1, _)), background): &AsrLensed| {
                    (DataWrapper(asr_1), background)
                },
            )))
            .with_child(prayer_row().lens(lens_map_get(
                |&(DataWrapper((_, asr_2)), background): &AsrLensed| {
                    (DataWrapper(asr_2.unwrap()), background)
                },
            ))),
        prayer_row().lens(lens_map_get(
            |&(DataWrapper((asr, _)), background): &AsrLensed| (DataWrapper(asr), background),
        )),
    )
    .background(background_painter())
    .lens(AppState::lens_asr());

    let now_remaining = Label::new(|(data, _): &(String, bool), _: &Env| data.clone())
        .with_text_size(15.0)
        .env_scope(|env, (_, critical)| {
            if *critical {
                env.set(theme::TEXT_COLOR, env.get(color::CLOSE_ACTIVE))
            }
        })
        .lens(AppState::lens_remaining())
        .controller(RemainingTimeController);

    let buttons = Flex::<AppState>::row()
        .with_child(
            Button::from_label(
                utils::Icon::Close
                    .label(FontWeight::REGULAR)
                    .with_text_size(16.0),
            )
            .env_scope(|env, _| {
                env.set(theme::BUTTON_LIGHT, env.get(color::CLOSE_HOT));
                env.set(theme::DISABLED_BUTTON_LIGHT, env.get(color::CLOSE_ACTIVE));
            })
            .on_click(|_, _, _| druid::Application::global().quit()),
        )
        .with_default_spacer()
        .with_child(
            Button::from_label(
                utils::Icon::Info
                    .label(FontWeight::REGULAR)
                    .with_text_size(16.0),
            )
            .on_click(|ctx, _, _| {
                ctx.new_window(
                    WindowDesc::new(ui_about::about_root())
                        .set_level(WindowLevel::Modal(ctx.window().clone()))
                        .show_titlebar(false)
                        .show_in_taskbar(false)
                        .set_always_on_top(true)
                        .resizable(false)
                        .window_size(ui_main::size::MAIN_WINDOW),
                )
            }),
        )
        .with_flex_spacer(1.0)
        .with_child(
            Button::from_label(
                utils::Icon::Settings
                    .label(FontWeight::REGULAR)
                    .with_text_size(16.0),
            )
            .on_click(|ctx, _, _| {
                ctx.new_window(
                    WindowDesc::new(ui_settings::settings_root().lens(lens!(AppState, config)))
                        .set_level(WindowLevel::Modal(ctx.window().clone()))
                        .set_position(size::CORNER_BUFFER.to_vec2().to_point())
                        .show_titlebar(false)
                        .show_in_taskbar(false)
                        .resizable(false)
                        .window_size(ui_settings::WINDOW_SIZE),
                )
            }),
        );

    Flex::column()
        .with_flex_spacer(1.0)
        .with_child(title)
        .with_default_spacer()
        .with_child(waqt_row(Prayer::Fajr))
        .with_child(
            prayer_row()
                .background(background_painter())
                .lens(AppState::lens_sunrise()),
        )
        .with_child(
            prayer_row()
                .background(background_painter())
                .lens(AppState::lens_dhuhr()),
        )
        .with_child(asr_row)
        .with_child(waqt_row(Prayer::Maghrib))
        .with_child(waqt_row(Prayer::Isha))
        .with_child(waqt_row(Prayer::Qiyam))
        .with_default_spacer()
        .with_child(now_remaining)
        .with_flex_spacer(1.0)
        .with_child(buttons)
        .center()
        .padding(16.0)
        .controller(RootController)
        .env_scope(|env, app_state| app_state.config.apply_appearance_to_env(env))
}

fn waqt_row(prayer: Prayer) -> impl Widget<AppState> {
    prayer_row()
        .background(background_painter())
        .lens(AppState::lens_prayer(prayer))
}

fn prayer_row() -> impl Widget<PrayerLensed> {
    Flex::row()
        .with_flex_child(
            Label::new(|data: &PrayerLensed, _: &Env| data.0 .0 .1.into())
                .with_text_size(18.0)
                .align_right(),
            FlexParams::new(1.0, CrossAxisAlignment::End),
        )
        .with_default_spacer()
        .with_flex_child(
            Label::new(move |data: &PrayerLensed, _: &Env| {
                data.0
                     .0
                     .0
                    .with_timezone(&chrono::Local)
                    .format("%I:%M %p")
                    .to_string()
            })
            .with_text_size(18.0)
            .align_left(),
            FlexParams::new(1.0, CrossAxisAlignment::Start),
        )
        .padding(2.0)
}

fn background_painter<T>() -> widget::Painter<(T, Option<(f64, bool)>)> {
    widget::Painter::new(|ctx, data: &(T, Option<(f64, bool)>), env| {
        if let Some((elapsed_fraction, critical)) = data.1 {
            let size = ctx.size();
            let elapsed_part = kurbo::RoundedRect::new(
                0.0,
                0.0,
                size.width * elapsed_fraction,
                size.height,
                kurbo::RoundedRectRadii::new(
                    size::ACTIVE_CORNER_RADIUS,
                    0.0,
                    0.0,
                    size::ACTIVE_CORNER_RADIUS,
                ),
            );
            let remaining_part = kurbo::RoundedRect::new(
                size.width * elapsed_fraction,
                0.0,
                size.width,
                size.height,
                kurbo::RoundedRectRadii::new(
                    0.0,
                    size::ACTIVE_CORNER_RADIUS,
                    size::ACTIVE_CORNER_RADIUS,
                    0.0,
                ),
            );

            ctx.fill(
                elapsed_part,
                &env.get(if critical {
                    color::ELAPSED_CRITICAL
                } else {
                    color::ELAPSED_OKAY
                }),
            );

            ctx.fill(remaining_part, &env.get(color::REMAINING));

            ctx.stroke(
                size.to_rounded_rect(size::ACTIVE_CORNER_RADIUS + 2.0),
                &env.get(theme::FOREGROUND_DARK),
                2.0,
            );
        }
    })
}

pub struct AppDelegate(pub Option<WindowId>);

impl druid::AppDelegate<AppState> for AppDelegate {
    fn focus(
        &mut self,
        _: &mut druid::DelegateCtx,
        id: druid::WindowId,
        handle: druid::WindowHandle,
        focus: bool,
        _: &mut AppState,
        _: &Env,
    ) {
        if !focus {
            if id == self.0.unwrap() {
                let raw_handle =
                    extract!(handle.raw_window_handle(), RawWindowHandle::Windows(h) => h);
                unsafe {
                    winuser::ShowWindow(std::mem::transmute(raw_handle.hwnd), winuser::SW_HIDE);
                }
            } else {
                handle.close()
            }
        }
    }

    fn window_added(
        &mut self,
        id: druid::WindowId,
        _: &mut AppState,
        _: &Env,
        ctx: &mut druid::DelegateCtx,
    ) {
        if self.0.is_none() {
            self.0 = Some(id);
        }
        ctx.submit_command(Command::new(selector::INIT, (), Target::Window(id)));
    }
}

pub struct RootController;

impl<W: Widget<AppState>> druid::widget::Controller<AppState, W> for RootController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(c) if c.is(selector::INIT) => {
                ctx.window().set_position(
                    druid_shell::Screen::get_monitors()
                        .into_iter()
                        .find(druid_shell::Monitor::is_primary)
                        .unwrap()
                        .virtual_work_rect()
                        .size()
                        .to_dp(ctx.window().get_scale().unwrap_or_default())
                        .to_vec2()
                        .to_point()
                        - ctx.window().get_size().to_vec2()
                        - size::CORNER_BUFFER.to_vec2(),
                );
            }
            Event::Command(c) if c.is(selector::SHOW) => unsafe {
                let hwnd = {
                    std::mem::transmute(extract!(ctx.window().raw_window_handle(), RawWindowHandle::Windows(h) => h).hwnd)
                };
                winuser::ShowWindow(hwnd, winuser::SW_SHOW);
                winuser::SetForegroundWindow(hwnd);
                winuser::SetActiveWindow(hwnd);
            },
            _ => child.event(ctx, event, data, env),
        }
    }
}

struct RemainingTimeController;

impl<W: Widget<AppState>> widget::Controller<AppState, W> for RemainingTimeController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        use std::time::Duration;

        match event {
            Event::Timer(_) => {
                tracing::debug!("Timer hit from `remaining`");

                use std::sync::atomic::{AtomicBool, Ordering};
                static FIRST_TIMER_HIT: AtomicBool = AtomicBool::new(true);
                if FIRST_TIMER_HIT.fetch_and(false, Ordering::Relaxed) {
                    ctx.request_timer(Duration::from_secs(60));
                }

                let now = Utc::now();
                if now.hour() == 0 && now.minute() == 0 && now.second() == 0 {
                    data.prayers = PrayerTimes::calculate(
                        now.date().naive_utc(),
                        data.config.coordinates,
                        data.config.method.get_parameters(),
                    )
                    .unwrap();
                }

                ctx.request_update();
            }
            _ => child.event(ctx, event, data, env),
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        match event {
            LifeCycle::BuildFocusChain => {
                use std::time::Duration;

                let now = Utc::now();
                ctx.request_timer(
                    Duration::from_secs(60 - now.second() as u64)
                        - Duration::from_nanos(now.nanosecond() as _),
                );
            }
            _ => child.lifecycle(ctx, event, data, env),
        }
    }
}
