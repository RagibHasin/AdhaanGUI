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

mod ui {
    use druid::{
        lens::Identity,
        widget::{Label, LabelText},
        Data, FontDescriptor, FontFamily, FontWeight, Lens, LensExt, LocalizedString,
    };

    pub fn lens_map_get<T, U>(lens_map: impl Fn(&T) -> U) -> impl Lens<T, U> {
        Identity.map(lens_map, |_, _| {})
    }

    pub fn localized_label<T>(label: &'static str) -> LabelText<T> {
        LocalizedString::new(label).with_placeholder(label).into()
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DataWrapper<T: PartialEq + Clone>(pub T);

    impl<T: PartialEq + Clone + 'static> Data for DataWrapper<T> {
        fn same(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

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
