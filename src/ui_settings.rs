use druid::{
    lens,
    lens::Identity,
    theme,
    widget::{
        prelude::*, Checkbox, CrossAxisAlignment, Flex, Label, Parse, SizedBox, Switch, TextBox,
    },
    LensExt, TextAlignment, WidgetExt,
};
use druid_widget_nursery::{prism, DropdownSelect, MultiCheckbox, MultiRadio, TitleBar};
use extract::try_extract;

use crate::{utils::localized_label, widgets::button::Button, *};

pub const WINDOW_SIZE: Size = Size::new(600.0, 750.0);

const SECTION_TITLE_SIZE: f64 = 15.0;
const LABEL_COLUMN_WIDTH: f64 = 100.0;
pub fn settings_root() -> impl Widget<Config> {
    let appearance_grp_label = TitleBar::new(
        Label::new(localized_label("Appearance"))
            .with_text_alignment(TextAlignment::Start)
            .with_font(theme::UI_FONT_BOLD)
            .with_text_size(SECTION_TITLE_SIZE)
            .expand_width(),
    );
    let font = Flex::row()
        .with_child(Label::new(localized_label("Font")).fix_width(LABEL_COLUMN_WIDTH))
        .with_flex_spacer(1.0)
        .with_flex_child(
            TextBox::new()
                .with_text_alignment(TextAlignment::End)
                .expand_width()
                .lens(lens!(Config, font)),
            1.0,
        );
    let dark_mode = Flex::row()
        .with_child(Label::new(localized_label("Dark mode")).fix_width(LABEL_COLUMN_WIDTH))
        .with_flex_spacer(1.0)
        .with_child(Switch::new().lens(lens!(Config, dark_mode)));

    let available_methods = {
        use config::AvailableMethods::*;
        [
            (localized_label("Dubai"), Dubai),
            (localized_label("Egyptian"), Egyptian),
            (localized_label("Karachi"), Karachi),
            (localized_label("Kuwait"), Kuwait),
            (localized_label("Muslim World League"), MuslimWorldLeague),
            (
                localized_label("Islamic Society North America"),
                NorthAmerica,
            ),
            (localized_label("Qatar"), Qatar),
            (localized_label("Singapore"), Singapore),
            (localized_label("Umm Al Qura University, Makkah"), UmmAlQura),
            (
                localized_label("Moonsighting Committee"),
                MoonsightingCommittee,
            ),
            (
                localized_label("Moonsighting Committee with red Isha"),
                MoonsightingCommitteeRedIsha,
            ),
            (
                localized_label("Moonsighting Committee with white Isha"),
                MoonsightingCommitteeWhiteIsha,
            ),
        ]
    };

    let calculation_grp_label = Label::new(localized_label("Calculation"))
        .with_text_alignment(TextAlignment::Start)
        .with_font(theme::UI_FONT_BOLD)
        .with_text_size(SECTION_TITLE_SIZE)
        .expand_width();
    let method = Flex::row()
        .with_child(Label::new(localized_label("Method")).fix_width(LABEL_COLUMN_WIDTH))
        .with_flex_spacer(1.0)
        .with_child(DropdownSelect::new(available_methods).lens(lens!(Config, method)));
    let location = Flex::row()
        .with_child(Label::new(localized_label("Location")).fix_width(LABEL_COLUMN_WIDTH))
        .with_default_spacer()
        .with_flex_child(
            TextBox::new()
                .with_text_alignment(TextAlignment::End)
                .expand_width()
                .lens(Identity.map(
                    |config: &Config| config.location_name.clone().unwrap_or_default(),
                    |config, input| {
                        config.location_name = if input.is_empty() { None } else { Some(input) }
                    },
                )),
            1.0,
        )
        .with_default_spacer()
        .with_flex_child(
            Label::new(localized_label("Latitude")), //.with_text_alignment(TextAlignment::End), // .fix_width(LABEL_COLUMN_WIDTH),
            0.8,
        )
        .with_default_spacer()
        .with_flex_child(
            Parse::new(
                TextBox::new()
                    .with_text_alignment(TextAlignment::End)
                    .expand_width(),
            )
            .lens(Identity.map(
                |config: &Config| Some(config.coordinates.latitude),
                |config, input| {
                    config.coordinates.latitude = input.unwrap_or(KAABA_COORDINATES.latitude)
                },
            )),
            1.0,
        )
        .with_default_spacer()
        .with_flex_child(
            Label::new(localized_label("Longitude")), //.with_text_alignment(TextAlignment::End), // .fix_width(LABEL_COLUMN_WIDTH),
            0.8,
        )
        .with_default_spacer()
        .with_flex_child(
            Parse::new(
                TextBox::new()
                    .with_text_alignment(TextAlignment::End)
                    .expand_width(),
            )
            .lens(Identity.map(
                |config: &Config| Some(config.coordinates.longitude),
                |config, input| {
                    config.coordinates.longitude = input.unwrap_or(KAABA_COORDINATES.longitude)
                },
            )),
            1.0,
        );
    let critical_at = Flex::row()
        .with_child(Label::new(localized_label("Warn")).fix_width(LABEL_COLUMN_WIDTH))
        .with_default_spacer()
        .with_child(Label::new(localized_label("when")))
        .with_child(Parse::new(
            TextBox::new()
                .with_text_alignment(TextAlignment::End)
                .fix_width(LABEL_COLUMN_WIDTH / 2.0),
        ))
        .with_child(Label::new(|data: &Option<u8>, _: &Env| {
            match *data {
                None | Some(0 | 1) => "minute remain",
                _ => "minutes remain",
            }
            .into()
        }))
        .with_flex_spacer(1.0)
        .lens(Identity.map(
            |config: &Config| Some(config.critical_at),
            |config, input| config.critical_at = input.unwrap_or(0),
        ));
    let ishraq = Flex::row()
        .with_child(Label::new(localized_label("Ishraq")).fix_width(LABEL_COLUMN_WIDTH))
        .with_default_spacer()
        .with_child(
            MultiCheckbox::new(
                "Enabled",
                Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(
                        Flex::row()
                            .with_child(
                                Label::new(localized_label("Starts after"))
                                    .fix_width(LABEL_COLUMN_WIDTH),
                            )
                            .with_child(
                                Parse::new(
                                    TextBox::new()
                                        .with_text_alignment(TextAlignment::End)
                                        .fix_width(LABEL_COLUMN_WIDTH / 2.0),
                                )
                                .lens(Identity.map(
                                    |ishraq: &(u8, u8)| Some(ishraq.0),
                                    |ishraq, input| {
                                        ishraq.0 = input.unwrap_or(DEFAULT_ISHRAQ_VALUE.0)
                                    },
                                )),
                            )
                            .with_child(Label::new(|ishraq: &(u8, u8), _: &Env| {
                                match ishraq.0 {
                                    0 | 1 => "minute after sunrise",
                                    _ => "minutes after sunrise",
                                }
                                .into()
                            })),
                    )
                    .with_default_spacer()
                    .with_child(
                        Flex::row()
                            .with_child(
                                Label::new(localized_label("Ends before"))
                                    .fix_width(LABEL_COLUMN_WIDTH),
                            )
                            .with_child(
                                Parse::new(
                                    TextBox::new()
                                        .with_text_alignment(TextAlignment::End)
                                        .fix_width(LABEL_COLUMN_WIDTH / 2.0),
                                )
                                .lens(Identity.map(
                                    |ishraq: &(u8, u8)| Some(ishraq.1),
                                    |ishraq, input| {
                                        ishraq.1 = input.unwrap_or(DEFAULT_ISHRAQ_VALUE.1)
                                    },
                                )),
                            )
                            .with_child(Label::new(|ishraq: &(u8, u8), _: &Env| {
                                match ishraq.0 {
                                    0 | 1 => "minute before midday",
                                    _ => "minutes before midday",
                                }
                                .into()
                            })),
                    ),
                (0, 0),
            )
            .with_space(0.0)
            .with_indent(20.0)
            .lens(lens!(Config, ishraq)),
        )
        .with_flex_spacer(1.0);
    let asr = Flex::row()
        .with_child(Label::new(localized_label("Dhuhr-Asr behavior")).fix_width(LABEL_COLUMN_WIDTH))
        .with_default_spacer()
        .with_child(
            Flex::column().cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(MultiRadio::new(
                    "Dhuhr ends at Asr awwal, naturally Asr begins then",
                    SizedBox::empty(),
                    (),
                    prism::Closures(
                        |asr: &AsrConfig| try_extract!(*asr, AsrConfig::DhuhrEndsAtAsrAwwal => ()),
                        |asr: &mut AsrConfig, _| *asr = AsrConfig::DhuhrEndsAtAsrAwwal,
                    ),
                ))
                .with_child(MultiRadio::new(
                    "Dhuhr ends at Asr thaani, but Asr begins at Asr awwal",
                    SizedBox::empty(),
                    (),
                    prism::Closures(
                        |asr: &AsrConfig| try_extract!(*asr, AsrConfig::DhuhrEndsAtAsrThaaniButAsrStartsAtAsrAwwal => ()),
                        |asr: &mut AsrConfig, _| *asr = AsrConfig::DhuhrEndsAtAsrThaaniButAsrStartsAtAsrAwwal,
                    ),
                ))
                .with_child(MultiRadio::new(
                    "Dhuhr ends at Asr thaani, naturally Asr begins then",
                    Checkbox::new(localized_label("Show both Asr")),
                    true,
                    prism::Closures(
                        |asr: &AsrConfig| try_extract!(*asr, AsrConfig::AsrStartsAtAsrThaani(show_both) => show_both),
                        |asr: &mut AsrConfig, show_both| *asr = AsrConfig::AsrStartsAtAsrThaani(show_both),
                    ),
                ).with_space(0.0).with_indent(20.0)).align_left(),
        ).with_flex_spacer(1.0)
        .lens(lens!(Config, asr));

    let adjustments_grp_label = Label::new(localized_label("User time adjustments"))
        .with_text_alignment(TextAlignment::Start)
        .with_font(theme::UI_FONT_BOLD)
        .with_text_size(SECTION_TITLE_SIZE)
        .expand_width();
    let adj_fajr = adjustment("Fajr").lens(lens!(Config, user_adjustments.fajr));
    let adj_sunrise = adjustment("Sunrise").lens(lens!(Config, user_adjustments.sunrise));
    let adj_dhuhr = adjustment("Dhuhr").lens(lens!(Config, user_adjustments.dhuhr));
    let adj_asr = adjustment("Asr").lens(lens!(Config, user_adjustments.asr));
    let adj_maghrib = adjustment("Maghrib").lens(lens!(Config, user_adjustments.maghrib));
    let adj_isha = adjustment("Isha").lens(lens!(Config, user_adjustments.isha));

    let ok = Flex::row()
        .with_flex_spacer(1.0)
        .with_child(Button::new(localized_label("OK")).on_click(|ctx, _, _| ctx.window().close()));

    Flex::column()
        .with_default_spacer()
        .with_child(appearance_grp_label)
        .with_default_spacer()
        .with_child(font)
        .with_default_spacer()
        .with_child(dark_mode)
        .with_default_spacer()
        // appearance done
        .with_default_spacer()
        //
        .with_default_spacer()
        .with_child(calculation_grp_label)
        .with_default_spacer()
        .with_child(method)
        .with_default_spacer()
        .with_child(location)
        .with_default_spacer()
        .with_child(critical_at)
        .with_default_spacer()
        .with_child(ishraq)
        .with_default_spacer()
        .with_child(asr)
        .with_default_spacer()
        // calculation done
        .with_default_spacer()
        //
        .with_default_spacer()
        .with_child(adjustments_grp_label)
        .with_default_spacer()
        .with_child(adj_fajr)
        .with_default_spacer()
        .with_child(adj_sunrise)
        .with_default_spacer()
        .with_child(adj_dhuhr)
        .with_default_spacer()
        .with_child(adj_asr)
        .with_default_spacer()
        .with_child(adj_maghrib)
        .with_default_spacer()
        .with_child(adj_isha)
        .with_default_spacer()
        // user adjustments done
        .with_flex_spacer(1.0)
        .with_child(ok)
        .align_left()
        .padding(16.0)
        .env_scope(|env, config| config.apply_appearance_to_env(env))
}

fn adjustment(prayer: &'static str) -> Flex<i64> {
    Flex::row()
        .with_child(Label::new(localized_label(prayer)).fix_width(LABEL_COLUMN_WIDTH))
        .with_default_spacer()
        .with_child(
            Parse::new(
                TextBox::new()
                    .with_text_alignment(TextAlignment::End)
                    .fix_width(LABEL_COLUMN_WIDTH / 2.0),
            )
            .lens(Identity.map(
                |adjustment: &i64| Some(*adjustment),
                |adjustment, input| *adjustment = input.unwrap_or_default(),
            )),
        )
        .with_child(Label::new(|adjustment: &i64, _: &Env| {
            match adjustment {
                0 | 1 => "minute",
                _ => "minutes",
            }
            .into()
        }))
        .with_flex_spacer(1.0)
}
