use adhaan::*;
use chrono::{DateTime, Duration, Utc};
use druid::Lens;

use crate::{
    config::{AsrConfig, Config},
    utils::{lens_map_get, DataWrapper},
};

#[derive(Clone, druid::Data)]
pub struct AppState {
    pub config: Config,

    #[data(same_fn = "PartialEq::eq")]
    pub prayers: PrayerTimes,
}

impl AppState {
    pub fn prayer_next(&self, prayer: Prayer) -> Prayer {
        use Prayer::*;

        match prayer {
            Yesterday => QiyamYesterday,
            QiyamYesterday => Fajr,
            Fajr => Sunrise,
            Sunrise => Dhuhr,
            Dhuhr if self.config.asr == AsrConfig::DhuhrEndsAtAsrAwwal => AsrAwwal,
            Dhuhr => AsrThaani,
            AsrAwwal | AsrThaani => Maghrib,
            Maghrib => Isha,
            Isha => Qiyam,
            Qiyam => Tomorrow,
            Tomorrow => unreachable!(),
        }
    }

    pub fn adjustment_for(&self, prayer: Prayer) -> Duration {
        use Prayer::*;

        Duration::minutes(match prayer {
            Fajr => self.config.user_adjustments.fajr,
            Sunrise => self.config.user_adjustments.sunrise,
            Dhuhr => self.config.user_adjustments.dhuhr,
            AsrAwwal | AsrThaani => self.config.user_adjustments.asr,
            Maghrib => self.config.user_adjustments.maghrib,
            Isha => self.config.user_adjustments.isha,
            _ => 0,
        })
    }

    pub fn label_of(&self, prayer: Prayer) -> &'static str {
        use Prayer::*;
        match prayer {
            Yesterday => "Isha",
            QiyamYesterday => "Qiyam",
            Fajr => "Fajr",
            Sunrise if self.config.ishraq.is_some() => "Ishraq",
            Sunrise => "Sunrise",
            Dhuhr => "Dhuhr",
            AsrAwwal if self.config.asr == AsrConfig::DhuhrEndsAtAsrAwwal => "Asr",
            AsrAwwal =>
                // if self.config.asr == AsrConfig::DhuhrEndsAtAsrThaaniButAsrStartsAtAsrAwwal
                //     || self.config.asr == AsrConfig::AsrStartsAtAsrThaani(true) =>
            {
                "Asr awwal"
            }
            AsrThaani if self.config.asr == AsrConfig::AsrStartsAtAsrThaani(false) => "Asr",
            AsrThaani => "Asr thaani",
            Maghrib => "Maghrib",
            Isha => "Isha",
            Qiyam => "Qiyam",
            Tomorrow => "Fajr",
        }
    }

    pub fn adjusted_time_of(&self, prayer: Prayer) -> DateTime<Utc> {
        self.prayers.time_of(prayer).unwrap() + self.adjustment_for(prayer)
    }

    pub fn lens_prayer(prayer: Prayer) -> impl Lens<Self, PrayerLensed> {
        lens_map_get(move |data: &Self| {
            let starts_at = data.adjusted_time_of(prayer);
            let now = Utc::now();
            let elapsed_critical = (prayer == data.prayers.prayer_at(now)).then(|| {
                let end = data.prayers.time_of(data.prayer_next(prayer)).unwrap();
                let remaining = end - now;
                let remaining_fraction =
                    remaining.num_seconds() as f64 / (end - starts_at).num_seconds() as f64;
                (
                    1.0 - remaining_fraction,
                    remaining.num_minutes() < data.config.critical_at as _,
                )
            });
            (
                DataWrapper((starts_at, data.label_of(prayer))),
                elapsed_critical,
            )
        })
    }

    pub fn lens_sunrise() -> impl Lens<Self, PrayerLensed> {
        lens_map_get(|data: &Self| {
            let starts_at = data.adjusted_time_of(Prayer::Sunrise);

            if let Some((after_sunrise, zawal)) = data.config.ishraq {
                let starts_at = starts_at + chrono::Duration::minutes(after_sunrise as _);
                let ends_at = data.prayers.time_of(Prayer::Dhuhr).unwrap()
                    - chrono::Duration::minutes(zawal as _);
                let now = Utc::now();

                let elapsed_critical = (starts_at..=ends_at).contains(&now).then(|| {
                    let remaining = ends_at - now;
                    let remaining_fraction =
                        remaining.num_seconds() as f64 / (ends_at - starts_at).num_seconds() as f64;
                    (
                        1.0 - remaining_fraction,
                        remaining.num_minutes() < data.config.critical_at as _,
                    )
                });

                return (
                    DataWrapper((starts_at, data.label_of(Prayer::Sunrise))),
                    elapsed_critical,
                );
            }

            (
                DataWrapper((starts_at, data.label_of(Prayer::Sunrise))),
                None,
            )
        })
    }

    pub fn lens_dhuhr() -> impl Lens<Self, PrayerLensed> {
        lens_map_get(|data: &Self| {
            let start = data.adjusted_time_of(Prayer::Dhuhr);
            let end = data
                .prayers
                .time_of(if data.config.asr == AsrConfig::DhuhrEndsAtAsrAwwal {
                    Prayer::AsrAwwal
                } else {
                    Prayer::AsrThaani
                })
                .unwrap();
            let now = Utc::now();

            let elapsed_critical = (start..=end).contains(&now).then(|| {
                let remaining = end - now;
                let remaining_fraction =
                    remaining.num_seconds() as f64 / (end - start).num_seconds() as f64;
                (
                    1.0 - remaining_fraction,
                    remaining.num_minutes() < data.config.critical_at as _,
                )
            });

            (
                DataWrapper((start, data.label_of(Prayer::Dhuhr))),
                elapsed_critical,
            )
        })
    }

    pub fn lens_asr() -> impl Lens<Self, AsrLensed> {
        lens_map_get(|data: &Self| {
            let start_1 = data.adjusted_time_of(Prayer::AsrAwwal);
            let start_2 = data.adjusted_time_of(Prayer::AsrThaani);

            let start = if let AsrConfig::AsrStartsAtAsrThaani(_) = data.config.asr {
                start_2
            } else {
                start_1
            };

            let start_2 = match data.config.asr {
                AsrConfig::DhuhrEndsAtAsrAwwal | AsrConfig::AsrStartsAtAsrThaani(false) => None,
                _ => Some((start_2, data.label_of(Prayer::AsrThaani))),
            };

            let end = data.prayers.time_of(Prayer::Maghrib).unwrap();
            let now = Utc::now();
            let elapsed_critical = (start..=end).contains(&now).then(|| {
                let remaining = end - now;
                let remaining_fraction =
                    remaining.num_seconds() as f64 / (end - start).num_seconds() as f64;
                (
                    1.0 - remaining_fraction,
                    remaining.num_minutes() < data.config.critical_at as _,
                )
            });
            (
                DataWrapper(((start_1, data.label_of(Prayer::AsrAwwal)), start_2)),
                elapsed_critical,
            )
        })
    }

    pub fn lens_remaining() -> impl Lens<Self, (String, bool)> {
        lens_map_get(|data: &Self| {
            let now = Utc::now();
            let prayer_now = data.prayers.prayer_at(now);

            if prayer_now == Prayer::Yesterday {
                return data.make_str_remaining_in_waqt_labeled(
                    prayer_now,
                    data.prayers.time_of(Prayer::QiyamYesterday).unwrap(),
                    now,
                );
            }

            let starts_at = data.adjusted_time_of(prayer_now);

            if prayer_now == Prayer::Sunrise {
                return if let Some((after_sunrise, zawal)) = data.config.ishraq {
                    let starts_at = starts_at + chrono::Duration::minutes(after_sunrise as _);
                    let dhuhr_starts_at = data.adjusted_time_of(Prayer::Dhuhr);
                    let ends_at = data.prayers.time_of(Prayer::Dhuhr).unwrap()
                        - chrono::Duration::minutes(zawal as _);

                    if now < starts_at {
                        data.make_str_remaining_to_waqt(Prayer::Sunrise, starts_at, now)
                    } else if now > ends_at {
                        data.make_str_remaining_to_waqt(Prayer::Dhuhr, dhuhr_starts_at, now)
                    } else {
                        data.make_str_remaining_in_waqt(ends_at, now)
                    }
                } else {
                    data.make_str_remaining_to_waqt(
                        Prayer::Dhuhr,
                        data.prayers.time_of(Prayer::Dhuhr).unwrap(),
                        now,
                    )
                };
            }

            let ends_with_prayer = match prayer_now {
                Prayer::QiyamYesterday
                | Prayer::Fajr
                | Prayer::Maghrib
                | Prayer::Isha
                | Prayer::Qiyam => data.prayer_next(prayer_now),
                Prayer::Dhuhr if data.config.asr == AsrConfig::DhuhrEndsAtAsrAwwal => {
                    Prayer::AsrAwwal
                }
                Prayer::Dhuhr => Prayer::AsrThaani,
                Prayer::AsrAwwal
                    if data.config.asr == AsrConfig::DhuhrEndsAtAsrThaaniButAsrStartsAtAsrAwwal =>
                {
                    return data.make_str_remaining_in_waqt_labeled(
                        Prayer::Dhuhr,
                        data.prayers.time_of(Prayer::AsrThaani).unwrap(),
                        now,
                    );
                }
                Prayer::AsrAwwal | Prayer::AsrThaani => Prayer::Maghrib,
                Prayer::Yesterday | Prayer::Sunrise | Prayer::Tomorrow => unreachable!(),
            };

            if now < starts_at {
                data.make_str_remaining_to_waqt(prayer_now, starts_at, now)
            } else {
                data.make_str_remaining_in_waqt(
                    data.prayers.time_of(ends_with_prayer).unwrap(),
                    now,
                )
            }
        })
    }

    fn make_str_remaining_in_waqt(
        &self,
        ends_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> (String, bool) {
        let time_remaining = ends_at - now;
        let mins_remaining = time_remaining.num_minutes() % 60;
        let plural_mins = if mins_remaining == 0 || mins_remaining == 1 {
            ""
        } else {
            "s"
        };
        (
            match time_remaining.num_hours() {
                0 => format!("{} minute{} remaining", mins_remaining, plural_mins),
                1 => format!(
                    "1 hour and {} minute{} remaining",
                    mins_remaining, plural_mins
                ),
                n => format!(
                    "{} hours and {} minute{} remaining",
                    n, mins_remaining, plural_mins
                ),
            },
            time_remaining.num_minutes() < self.config.critical_at as _,
        )
    }

    fn make_str_remaining_in_waqt_labeled(
        &self,
        prayer: Prayer,
        ends_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> (String, bool) {
        let (text, critical) = self.make_str_remaining_in_waqt(ends_at, now);
        (format!("{} of {}", text, self.label_of(prayer)), critical)
    }

    fn make_str_remaining_to_waqt(
        &self,
        prayer: Prayer,
        starts_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> (String, bool) {
        let (text, critical) = self.make_str_remaining_in_waqt(starts_at, now);
        (format!("{} to {}", text, self.label_of(prayer)), critical)
    }
}

pub type PrayerLensed = (
    DataWrapper<(DateTime<Utc>, &'static str)>,
    Option<(f64, bool)>,
);

pub type AsrLensed = (
    DataWrapper<(
        (DateTime<Utc>, &'static str),
        Option<(DateTime<Utc>, &'static str)>,
    )>,
    Option<(f64, bool)>,
);
