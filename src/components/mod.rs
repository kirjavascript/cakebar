use crate::bar::Bar;
use crate::config::ConfigGroup;
use crate::wm;
use gtk::{Align, ContainerExt, OverlayExt, StyleContextExt, WidgetExt};
use {glib, gtk};

mod backlight;
// mod bandwidth;
// mod battery;
// mod clock;
// mod command_input;
// mod container;
// mod cpu;
// mod disk;
// mod dropdown;
// mod image;
// mod ip;
// mod memory;
// mod mode;
// mod script;
// mod tray;
// mod window;
// mod workspaces;

/// defines interface for components
pub trait Component {
    /// clean up any remaining timeouts, callbacks
    fn destroy(&self);
}

/// each component should call bar.add_component
pub fn load_component(config: ConfigGroup, bar: &mut Bar, container: &gtk::Box) {
    fn void(config: ConfigGroup, _: &mut Bar, _: &gtk::Box) {
        warn!("a valid type is required for #{}", config.name)
    }
    // decide which component to load
    (match config.get_str_or("type", "void") {
        "backlight" => backlight::Backlight::init,
        // "bandwidth" => bandwidth::Bandwidth::init,
        // "battery" => battery::Battery::init,
        // "clock" => clock::Clock::init,
        // "command_input" => command_input::CommandInput::init,
        // "container" => container::Container::init,
        // "cpu" => cpu::CPU::init,
        // "disk" => disk::Disk::init,
        // "dropdown" => dropdown::Dropdown::init,
        // "image" => image::Image::init,
        // "ip" => ip::IP::init,
        // "memory" => memory::Memory::init,
        // "mode" => mode::Mode::init,
        // "script" => script::Script::init,
        // "tray" => tray::Tray::init,
        // "window" => window::Window::init,
        // "workspaces" => workspaces::Workspaces::init,
        _ => void,
    })(config, bar, container);
}

pub fn init_widget<T>(widget: &T, config: &ConfigGroup, bar: &Bar, container: &gtk::Box)
where
    T: gtk::IsA<gtk::Widget> + gtk::IsA<gtk::Object> + glib::value::SetValue,
{
    // TODO: add EventBox
    // TODO: add wrapper
    // let wrapper = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    // wrapper.add(widget);
    // wrapper.show();
    // let widget = &wrapper;
    // set name
    widget.set_name(&config.name);
    // class
    let class_str = config.get_str_or("class", "void");
    if class_str != "void" {
        if let Some(ctx) = widget.get_style_context() {
            ctx.add_class(class_str);
        }
    }
    let is_fixed = config.get_bool_or("fixed", false);
    // set alignment
    let halign_str = config.get_str_or("halign", "void");
    if halign_str != "void" {
        WidgetExt::set_halign(widget, get_alignment(halign_str));
        if !is_fixed {
            WidgetExt::set_hexpand(widget, true);
        }
    }
    let valign_str = config.get_str_or("valign", "void");
    if valign_str != "void" {
        WidgetExt::set_valign(widget, get_alignment(valign_str));
        if !is_fixed {
            WidgetExt::set_vexpand(widget, true);
        }
    }
    // set layout type
    if is_fixed {
        bar.overlay.add_overlay(widget);
        if config.get_bool_or("pass-through", true) {
            bar.overlay.set_overlay_pass_through(widget, true);
        }
    } else {
        container.add(widget);
    }
}

fn get_alignment(align: &str) -> Align {
    match align {
        "start" => Align::Start,
        "end" => Align::End,
        "center" | "centre" => Align::Center,
        "fill" => Align::Fill,
        _ => Align::Baseline,
    }
}

// use bar::Bar;
// use components::Component;
// use crate::config::ConfigGroup;
// use gtk::prelude::*;
//
// pub struct Template {
// }
//
// impl Component for Template {
//     fn destroy(&self) {
//     }
// }
//
// impl Template {
//     pub fn init(config: ConfigGroup, bar: &mut Bar, container: &gtk::Box) {
//         //super::init_widget(&entry, &config, bar, container);
//
//         bar.add_component(Box::new(Template));
//     }
// }
