use std::path::PathBuf;

pub fn appdata_dir() -> PathBuf {
    let mut appdata_dir = if cfg!(debug_assertions) {
        PathBuf::new()
    } else if cfg!(windows) {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut home = PathBuf::from(std::env::var_os("HOME").unwrap());
                home.push(".config");
                home
            })
    };
    appdata_dir.push("AdhaanGUI");
    appdata_dir
}

pub fn config_path() -> PathBuf {
    let mut config_path = appdata_dir();
    config_path.push("config.toml");
    config_path
}

pub fn theme_dir() -> PathBuf {
    let mut theme_dir = appdata_dir();
    theme_dir.push("themes");
    theme_dir
}

#[derive(Debug, Clone, druid::Data)]
pub struct DataDuration(#[data(same_fn = "PartialEq::eq")] pub chrono::Duration);

mod ui {
    use druid::{widget::Label, Data, FontDescriptor, FontFamily, FontWeight};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Icon {
        Location,
        Settings,
        Close,
        Info,
    }

    impl Icon {
        #[cfg(windows)]
        pub fn as_str(self) -> &'static str {
            let ver_major = unsafe { *(0x7FFE026Cusize as *const u32) };
            if ver_major < 10 {
                panic!("Minimum supported version in Windows 10");
            }

            match self {
                Icon::Location => "\u{e81d}",
                Icon::Settings => "\u{e713}",
                // Icon::Close => "\u{ef2c}",
                Icon::Close => "\u{e8bb}",
                Icon::Info => "\u{e946}",
            }
        }

        #[cfg(windows)]
        pub fn label<T: Data>(self, weight: FontWeight) -> Label<T> {
            let ver_build = unsafe { *(0x7FFE0260usize as *const u32) };

            Label::new(self.as_str()).with_font(
                FontDescriptor::new(FontFamily::new_unchecked(if ver_build >= 22000 {
                    "Segoe Fluent Icons"
                } else {
                    "Segoe MDL2 Assets"
                }))
                .with_weight(weight),
            )
        }
    }
}

pub use ui::*;
