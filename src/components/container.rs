use super::{Component, Bar, ComponentConfig, Property};
use gtk::prelude::*;
use gtk::{Box, Orientation, Align};

pub struct Container {
}

impl Component for Container {
    fn init(container: &Box, config: &ComponentConfig, bar: &Bar) {
        if let Some(layout) = config.properties.get("layout") {

            // get spacing
            let spacing = config.get_int_or("spacing", 0) as i32;

            // get direction
            let direction = config.get_str_or("direction", "column");
            let direction = match direction {
                "column" | "horiz" => Orientation::Horizontal,
                _ => Orientation::Vertical,
            };

            // let direction = Orientation::Horizontal;

            let new_container = Box::new(direction, spacing);
            WidgetExt::set_name(&new_container, &config.name);
            super::layout_to_container(&new_container, layout, bar);
            container.add(&new_container);

            if false {
                new_container.set_hexpand(true);
                new_container.set_halign(Align::Fill);
            }
        }
    }
}