use std::process::Command;
use std::thread;

pub fn set_padding(is_top: bool, padding: i32) {
    let position = if is_top { "top" } else { "bottom" };

    thread::spawn(move || {
        Command::new("bspc")
            .arg("config")
            .arg(format!("{}_padding", position))
            .arg(format!("{}", padding))
            .output()
            .ok();
    });
}

use std::env;
use std::os::unix::net::{UnixStream};
use std::io::{Error};//, Write, Read

pub fn connect() -> Result<UnixStream, Error> {
// https://github.com/marionauta/bspc/blob/master/src/main.rs
//https://github.com/baskerville/bspwm/blob/336095739e2de94109e55e544c806770316c822c/doc/bspwm.1.asciidoc
    let stream_file = env::var("BSPWM_SOCKET")
        .unwrap_or("/tmp/bspwm{}_0_0-socket".to_string());

    // let mut stream = UnixStream::connect(stream_file.clone()).unwrap();
// stream.write(b"config top_padding 200\0").unwrap();
// let mut response = String::new();
// stream.read_to_string(&mut response).unwrap();
// println!("{}", response);

    UnixStream::connect(stream_file)
}
