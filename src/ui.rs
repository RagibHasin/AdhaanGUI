use druid::{
    lens, theme,
    widget::{self, prelude::*, CrossAxisAlignment, Flex, FlexParams, Label},
    FontWeight, HasRawWindowHandle, LensExt, LocalizedString, RawWindowHandle, Scalable, UnitPoint,
    WidgetExt,
};
use extract::*;

use winapi::um::winuser;

use crate::{widgets::button::Button, *};

/// Generates `Key`s based on module, line and column
/// ```
/// # use druid_widget_nursery::keys;
/// keys! {
///     /// height of the bar
///     BAR: usize,
/// }
/// ```
/// expands to
/// ```
/// # use druid::Key;
/// /// height of the bar
/// pub const BAR: Key<usize> = Key::new("path::to::module::BAR@0:0");
/// ```
#[macro_export]
macro_rules! keys {
        (
            $(
                $(#[$attr:meta])*
                $name:ident : $ty:ty
            ),* $(,)?
        ) => {
            $(
                $(#[$attr])*
                pub const $name: ::druid::Key<$ty> = ::druid::Key::new(concat!(
                    module_path!(),
                    "::",
                    stringify!($name),
                    "@",
                    line!(),
                    ":",
                    column!()
                ));
            )*
        };
    }

pub mod color {
    use druid::Color;

    keys! {
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
}

pub mod selector {
    use druid::Selector;

    pub const INIT: Selector<()> = Selector::new("init");
    pub const SHOW: Selector<()> = Selector::new("show");
}

pub static TRAY_ICON: &[u8] = include_bytes!("../resources/icon.ico");

pub fn main_root() -> impl Widget<AppState> {
    let location_name = widget::Maybe::or_empty(|| {
        Flex::row()
            //.cross_axis_alignment(CrossAxisAlignment::Baseline)
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

    let waqt_row = move |label, prayer| {
        Flex::row()
            .with_flex_child(
                Label::new(LocalizedString::new(label).with_placeholder(label))
                    .with_text_size(18.0)
                    .align_right(),
                FlexParams::new(1.0, CrossAxisAlignment::End),
            )
            .with_default_spacer()
            .with_flex_child(
                Label::new(move |data: &AppState, _: &Env| {
                    data.prayers
                        .time_of(prayer)
                        .with_timezone(&chrono::Local)
                        .format("%I:%M %p")
                        .to_string()
                })
                .with_text_size(18.0)
                .align_left(),
                FlexParams::new(1.0, CrossAxisAlignment::Start),
            )
            .padding(2.0)
            .background(widget::Painter::new(move |ctx, data: &AppState, env| {
                let now = chrono::Utc::now();
                if data.prayers.prayer_at(now) == Some(prayer) {
                    if let Some(next_prayer) = prayer.next() {
                        let begin = data.prayers.time_of(prayer);
                        let end = data.prayers.time_of(next_prayer);

                        let total = end - begin;
                        let remaining = end - now;
                        let elapsed = now - begin;
                        let elapsed_fraction =
                            elapsed.num_seconds() as f64 / total.num_seconds() as f64;

                        let size = ctx.size();
                        let elapsed_part = kurbo::RoundedRect::new(
                            0.0,
                            0.0,
                            size.width * elapsed_fraction,
                            size.height,
                            kurbo::RoundedRectRadii::new(4.0, 0.0, 0.0, 4.0),
                        );
                        let remaining_part = kurbo::RoundedRect::new(
                            size.width * elapsed_fraction,
                            0.0,
                            size.width,
                            size.height,
                            kurbo::RoundedRectRadii::new(0.0, 4.0, 4.0, 0.0),
                        );

                        ctx.fill(
                            elapsed_part,
                            &env.get(if remaining < chrono::Duration::minutes(5) {
                                color::ELAPSED_CRITICAL
                            } else {
                                color::ELAPSED_OKAY
                            }),
                        );

                        ctx.fill(remaining_part, &env.get(color::REMAINING));

                        ctx.stroke(
                            size.to_rounded_rect(4.0),
                            &env.get(theme::FOREGROUND_DARK),
                            2.0,
                        );
                    }
                }
            }))
    };

    let now_remaining = widget::Maybe::or_empty(|| {
        Flex::column().with_default_spacer().with_child(
            Label::new(|data: &utils::DataDuration, _: &Env| {
                let time_remaining = data.0;
                let mins_remaining = time_remaining.num_minutes() % 60;
                match time_remaining.num_hours() {
                    0 => format!("{} minutes remaining", mins_remaining),
                    1 => format!("1 hour and {} minutes remaining", mins_remaining),
                    n => format!("{} hours and {} minutes remaining", n, mins_remaining),
                }
            })
            .with_text_size(15.0)
            .env_scope(|env, data| {
                if data.0 < chrono::Duration::minutes(5) {
                    env.set(theme::TEXT_COLOR, env.get(color::CLOSE_ACTIVE))
                }
            }),
        )
    })
    .controller(RemainingTimeController)
    .lens(lens!(AppState, prayers).map(
        |prayers| {
            prayers
                .time_remaining(chrono::Utc::now())
                .map(utils::DataDuration)
        },
        |_, _| {},
    ));

    let buttons = Flex::<AppState>::row()
        .with_child(
            Button::from_label(
                utils::Icon::Close
                    .label(FontWeight::EXTRA_BLACK)
                    .with_text_size(16.0),
            )
            .with_hot_color(color::CLOSE_HOT)
            .with_active_color(color::CLOSE_ACTIVE)
            .on_click(|_, _, _| druid::Application::global().quit()),
        )
        .with_flex_spacer(1.0);
    // .with_child(
    //     Button::from_label(
    //         fluent_icon_label("\u{e713}", FontWeight::REGULAR).with_text_size(16.0),
    //     )
    //     .on_click(|_, _, _| eprintln!("SETTINGS CLICKED")),
    // );

    Flex::column()
        .with_flex_spacer(1.0)
        .with_child(title)
        .with_default_spacer()
        .with_child(waqt_row("Fajr", Prayer::Fajr))
        .with_child(waqt_row("Sunrise", Prayer::Sunrise))
        .with_child(waqt_row("Dhuhr", Prayer::Dhuhr))
        .with_child(waqt_row("Asr Awwal", Prayer::AsrAwwal))
        .with_child(waqt_row("Asr Thaani", Prayer::AsrThaani))
        .with_child(waqt_row("Maghrib", Prayer::Maghrib))
        .with_child(waqt_row("Isha", Prayer::Isha))
        .with_child(waqt_row("Qiyam", Prayer::Qiyam))
        .with_child(now_remaining)
        .with_flex_spacer(1.0)
        .with_child(buttons)
        .align_vertical(UnitPoint::CENTER)
        .padding(16.0)
        .controller(RootController)
}

pub struct AppDelegate;

impl druid::AppDelegate<AppState> for AppDelegate {
    fn focus(
        &mut self,
        _: &mut druid::DelegateCtx,
        _: druid::WindowId,
        handle: druid::WindowHandle,
        focus: bool,
        _: &mut AppState,
        _: &Env,
    ) {
        if !focus {
            let raw_handle = extract!(handle.raw_window_handle(), RawWindowHandle::Windows(h) => h);
            unsafe {
                winuser::ShowWindow(std::mem::transmute(raw_handle.hwnd), winuser::SW_HIDE);
            }
        }
    }

    fn window_added(
        &mut self,
        _: druid::WindowId,
        _: &mut AppState,
        _: &Env,
        ctx: &mut druid::DelegateCtx,
    ) {
        ctx.submit_command(selector::INIT)
    }
}

struct RootController;

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
                        - size::MAIN_WINDOW.to_vec2()
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

impl<W: Widget<Option<utils::DataDuration>>>
    druid::widget::Controller<Option<utils::DataDuration>, W> for RemainingTimeController
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Option<utils::DataDuration>,
        env: &Env,
    ) {
        use std::time::Duration;

        match event {
            Event::Timer(_) => {
                tracing::debug!("Timer hit");

                use std::sync::atomic::{AtomicBool, Ordering};
                static FIRST_TIMER_HIT: AtomicBool = AtomicBool::new(true);
                if FIRST_TIMER_HIT.fetch_and(false, Ordering::Relaxed) {
                    ctx.request_timer(Duration::from_secs(60));
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
        data: &Option<utils::DataDuration>,
        env: &Env,
    ) {
        match event {
            LifeCycle::BuildFocusChain => {
                use chrono::{Timelike, Utc};
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
