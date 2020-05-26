use os_info::Type;
use std::process::Command;

pub fn open(media: String) -> () {
    let open_command = match os_info::get().os_type() {
        Type::Macos => "open",
        _ => "xdg-open",
    };

    Command::new(open_command)
        .arg(media.clone())
        .status()
        .expect(&format!("Failed to open media: {}", media));
}
