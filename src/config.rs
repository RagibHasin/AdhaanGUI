use druid::Data;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Data)]
pub struct Config {
    pub font: String,
    pub dark_mode: bool,

    #[data(same_fn = "PartialEq::eq")]
    pub method: AvailableMethods,
    pub location_name: Option<String>,
    pub critical_at: u8,
    pub ishraq: Option<(u8, u8)>,
    pub asr: AsrConfig,

    #[serde(with = "remote_defs::Coordinates")]
    #[data(same_fn = "PartialEq::eq")]
    pub coordinates: adhaan::Coordinates,

    #[serde(with = "remote_defs::TimeAdjustment")]
    #[serde(default)]
    #[data(same_fn = "PartialEq::eq")]
    pub user_adjustments: adhaan::TimeAdjustment,
}

impl Config {
    pub fn load() -> anyhow::Result<Config> {
        let config_path = crate::utils::config_path();
        if config_path.exists() {
            Ok(toml::from_slice(&std::fs::read(config_path)?)?)
        } else {
            let config = Config::default();
            let config_toml = toml::to_string(&config).unwrap();
            std::fs::write(config_path, config_toml)?;
            Ok(config)
        }
    }

    pub fn apply_appearance_to_env(&self, env: &mut druid::Env) {
        use crate::ui_main::color;
        use druid::{theme, Color};

        let font_family =
            druid::FontDescriptor::new(druid::FontFamily::new_unchecked(self.font.as_str()));
        env.set(
            theme::UI_FONT_BOLD,
            font_family.clone().with_weight(druid::FontWeight::BOLD),
        );
        env.set(
            theme::UI_FONT_ITALIC,
            font_family.clone().with_style(druid::FontStyle::Italic),
        );
        env.set(theme::UI_FONT, font_family);

        if !self.dark_mode {
            env.set(theme::WINDOW_BACKGROUND_COLOR, Color::grey(0.84));
            env.set(theme::TEXT_COLOR, Color::BLACK);
            env.set(theme::BACKGROUND_DARK, Color::grey(0.82));
            env.set(theme::BACKGROUND_LIGHT, Color::grey(0.8));
            env.set(theme::FOREGROUND_DARK, Color::grey8(0x40));
            env.set(theme::FOREGROUND_LIGHT, Color::grey(0.16));
            env.set(theme::BUTTON_LIGHT, Color::grey(0.9));
            env.set(theme::DISABLED_BUTTON_LIGHT, Color::grey(0.8));

            env.set(color::ELAPSED_CRITICAL, Color::from_rgba32_u32(0xFFB492_FF));
            env.set(color::ELAPSED_OKAY, Color::from_rgba32_u32(0xFFDC61_FF));
            env.set(color::REMAINING, Color::from_rgba32_u32(0x00FFA6_FF));

            env.set(color::CLOSE_HOT, Color::rgb8(216, 44, 29));
            env.set(color::CLOSE_ACTIVE, Color::rgb8(196, 43, 28));
        } else {
            env.set(color::ELAPSED_CRITICAL, Color::from_rgba32_u32(0x7B2E15_FF));
            env.set(color::ELAPSED_OKAY, Color::from_rgba32_u32(0x735C00_FF));
            env.set(color::REMAINING, Color::from_rgba32_u32(0x006008_FF));

            env.set(color::CLOSE_HOT, Color::rgb8(196, 43, 28));
            env.set(color::CLOSE_ACTIVE, Color::rgb8(178, 42, 27));
        }
    }
}

pub const KAABA_COORDINATES: adhaan::Coordinates = adhaan::Coordinates {
    latitude: 21.422487,
    longitude: 39.826206,
};
pub const DEFAULT_ISHRAQ_VALUE: (u8, u8) = (15, 10);

impl Default for Config {
    fn default() -> Self {
        Config {
            font: "Segoe UI".into(),
            dark_mode: true,

            method: AvailableMethods::UmmAlQura,

            critical_at: 15,

            ishraq: Some(DEFAULT_ISHRAQ_VALUE),
            asr: AsrConfig::DhuhrEndsAtAsrAwwal,

            coordinates: KAABA_COORDINATES,
            location_name: Some("Kaaba".into()),

            user_adjustments: adhaan::TimeAdjustment::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Data)]
#[serde(tag = "mode", content = "show_both")]
pub enum AsrConfig {
    DhuhrEndsAtAsrAwwal,
    DhuhrEndsAtAsrThaaniButAsrStartsAtAsrAwwal,
    AsrStartsAtAsrThaani(bool),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Data)]
pub enum AvailableMethods {
    Dubai,
    Egyptian,
    Karachi,
    Kuwait,
    MuslimWorldLeague,
    NorthAmerica,
    Qatar,
    Singapore,
    UmmAlQura,
    MoonsightingCommittee,
    MoonsightingCommitteeRedIsha,
    MoonsightingCommitteeWhiteIsha,
}

impl AvailableMethods {
    pub fn get_parameters(self) -> adhaan::Parameters {
        use adhaan::{prominent_methods::*, Method};
        adhaan::Parameters::new(match self {
            Self::Dubai => &Dubai as &dyn Method,
            Self::Egyptian => &Egyptian as &dyn Method,
            Self::Karachi => &Karachi as &dyn Method,
            Self::Kuwait => &Kuwait as &dyn Method,
            Self::MuslimWorldLeague => &MuslimWorldLeague as &dyn Method,
            Self::NorthAmerica => &NorthAmerica as &dyn Method,
            Self::Qatar => &Qatar as &dyn Method,
            Self::Singapore => &Singapore as &dyn Method,
            Self::UmmAlQura => &UmmAlQura as &dyn Method,
            Self::MoonsightingCommittee => &MoonsightingCommittee as &dyn Method,
            Self::MoonsightingCommitteeRedIsha => &MoonsightingCommitteeRedIsha as &dyn Method,
            Self::MoonsightingCommitteeWhiteIsha => &MoonsightingCommitteeWhiteIsha as &dyn Method,
        })
    }
}

mod remote_defs {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "adhaan::TimeAdjustment")]
    pub struct TimeAdjustment {
        pub fajr: i64,
        pub sunrise: i64,
        pub dhuhr: i64,
        pub asr: i64,
        pub maghrib: i64,
        pub isha: i64,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "adhaan::Coordinates")]
    pub struct Coordinates {
        pub latitude: f64,
        pub longitude: f64,
    }
}
