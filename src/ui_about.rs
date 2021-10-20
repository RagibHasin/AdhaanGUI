use druid::{
    theme,
    widget::{prelude::*, CrossAxisAlignment, Flex, Label},
    Color, WidgetExt,
};

use crate::*;

pub fn about_root() -> impl Widget<AppState> {
    let link_color: Color = Color::rgb(0.2, 0.4, 0.9);

    let alhamdulillah = Label::new(utils::localized_label("Alhamdulillah")).with_text_size(18.0);
    let title = Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Baseline)
        .with_child(Label::new("Adhaan").with_text_size(18.0))
        .with_child(Label::new("GUI").with_font(theme::UI_FONT_BOLD))
        .with_child(Label::new("v0.1").with_font(theme::UI_FONT_ITALIC));
    let hadiya1 = Label::new("A hadiya from");
    let hadiya2 = Label::new("Muhammad Ragib Hasin");
    let github = Label::new("github.com/RagibHasin/AdhaanGUI")
        .with_text_color(link_color.clone())
        .on_click(|_, _, _| open::that("https://github.com/RagibHasin/AdhaanGUI").unwrap());
    let license1 = Label::new("Licensed to be free with");
    let license2 = Label::new("GNU AGPL 3.0")
        .with_text_color(link_color)
        .on_click(|_, _, _| open::that("https://www.gnu.org/licenses/agpl-3.0.en.html").unwrap());
    let suggestions = Label::new("Suggestions, feature requests and pull requests are welcome.")
        .with_text_alignment(druid::TextAlignment::Center)
        .with_line_break_mode(druid::widget::LineBreaking::WordWrap);
    let prayer = Label::new("May Allah guide us all to the righteous path.");
    let salam = Label::new("As-salamu 'alaykum");

    Flex::column()
        .with_flex_spacer(1.0)
        .with_child(alhamdulillah)
        .with_flex_spacer(1.0)
        .with_default_spacer()
        .with_child(title)
        .with_default_spacer()
        .with_child(hadiya1)
        .with_child(hadiya2)
        .with_default_spacer()
        .with_child(github)
        .with_default_spacer()
        .with_child(license1)
        .with_child(license2)
        .with_default_spacer()
        .with_child(suggestions)
        .with_default_spacer()
        .with_child(prayer)
        .with_default_spacer()
        .with_child(salam)
        .with_flex_spacer(1.0)
        .center()
        .padding(16.0)
        .controller(ui_main::RootController)
        .on_click(|ctx, _, _| ctx.window().close())
        .env_scope(|env, app_state| {
            app_state.config.apply_appearance_to_env(env);
            env.set(theme::UI_FONT, env.get(theme::UI_FONT).with_size(12.0))
        })
}
