#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("./resources/icon.ico")
        .set("FileDescription", "Adhaan GUI")
        .set("CompanyName", "LilAhad Labs")
        .set("LegalCopyright", "Copyright (C) 2021  Muhammad Ragib Hasin");
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {}
