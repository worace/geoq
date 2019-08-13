extern crate os_type;

use std::process::Command;

pub fn open(media: String) -> () {
    let open_command = match os_type::current_platform().os_type {
        os_type::OSType::OSX => "open",
        _ => "xdg-open"
    };

    Command::new(open_command)
        .arg(media.clone())
        .status()
        .expect(&format!("Failed to open media: {}", media));
}
