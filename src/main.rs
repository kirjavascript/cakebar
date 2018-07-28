extern crate glib;
extern crate cairo;
extern crate gio;
extern crate gtk;
extern crate gdk;
extern crate libc;
extern crate xcb;
extern crate gdk_sys;
extern crate libpulse_sys;
extern crate libpulse_simple_sys;
extern crate clap;
extern crate toml;
extern crate ansi_term;
extern crate i3ipc;
extern crate chrono;
extern crate systemstat;
extern crate probes;
extern crate sysinfo;
#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate bincode;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use gio::prelude::*;

use clap::{Arg, App};

#[macro_use]
mod macros;
mod util;
mod config;
mod bar;
mod components;
mod tray;
mod wm;

pub static NAME: &str = env!("CARGO_PKG_NAME");
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

fn init(application: &gtk::Application, config: &config::Config) {
    // components::equalizer::test();

    // load theme to screen
    match &config.theme {
        &Some(ref src) => wm::gtk::load_theme(src),
        &None => {/* default theme */},
    }

    let monitors = wm::gtk::get_monitor_geometry();
    let wm_util = wm::WMUtil::new();

    // load bars
    for bar_config in config.bars.iter() {
        let monitor_index = bar_config.get_int_or("monitor", 0);
        let monitor_option = monitors.get(monitor_index as usize);

        if let Some(monitor) = monitor_option {
            let _ = bar::Bar::new(
                &application,
                &config,
                &bar_config,
                &wm_util,
                monitor,
            );
        } else {
            warn!("no monitor at index {}", monitor_index);
        }
    }
}

fn main() {

    // CLI config

    let matches = App::new(NAME)
        .version(VERSION)
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Specify a config path")
             .takes_value(true))
        .arg(Arg::with_name("monitors")
             .short("m")
             .long("monitors")
             .help("Shows information about monitors"))
        .arg(Arg::with_name("tray")
             .short("t")
             .long("tray")
             .help("Loads system tray")
             .hidden(true))
        .get_matches();

    // show monitor debug
    if matches.is_present("monitors") {
        wm::gtk::show_monitor_debug();
        return ();
    }
    // load tray
    else if matches.is_present("tray") {
        tray::main();
        return ();
    }

    // get config

    let default_path = format!("{}/config.toml", util::get_config_dir());

    let config_path = matches.value_of("config").unwrap_or(&default_path);

    let config_res = config::parse_config(config_path);

    if let Ok(config) = config_res {

        // GTK application

        // check version
        if let Some(err) = gtk::check_version(3, 22, 0) {
            warn!("{} (requires 3.22+)", err);
        }

        let application = gtk::Application::new(
                format!("com.kirjava.{}", NAME).as_str(),
                gio::ApplicationFlags::NON_UNIQUE,
            )
            .expect("Initialization failed...");

        application.connect_startup(move |app| {
            init(&app, &config);
        });
        application.connect_activate(|_| {});

        application.run(&Vec::new()); // dont pass any arguments to GTK

    } else if let Err(msg) = config_res {
        error!("{}", msg);
    }


}
