use crate::bar::Bar;
use crate::components::Component;
use crate::config::ConfigGroup;
use gtk::prelude::*;
use gtk::Orientation;

pub struct Container {
    wrapper: gtk::Box,
}

impl Component for Container {
    fn destroy(&self) {
        self.wrapper.destroy();
    }
}

impl Container {
    pub fn init(config: ConfigGroup, bar: &mut Bar, container: &gtk::Box) {
        // get spacing
        let spacing = config.get_int_or("spacing", 0) as i32;

        // get direction
        let direction = match config.get_str_or("direction", "horizontal") {
            "horizontal" => Orientation::Horizontal,
            _ => Orientation::Vertical,
        };

        let wrapper = gtk::Box::new(direction, spacing);
        super::init_widget(&wrapper, &config, bar, container);
        wrapper.show();

        // load layout
        for name in config.get_string_vec("layout") {
            let config_opt = bar.wm_util.get_component_config(&name);
            if let Some(config) = config_opt {
                super::load_component(&bar.wm_util, config, bar, &wrapper);
            } else {
                warn!("missing component #{}", name);
            }
        }

        bar.add_component(Box::new(Container { wrapper }));
    }
}
