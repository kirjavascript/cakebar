use crate::bar::Bar;
use chrono::Local;
use crate::components::Component;
use crate::config::ConfigGroup;
use gtk;
use gtk::prelude::*;
use gtk::Label;
use crate::util::{SymbolFmt, Timer};

pub struct Clock {
    label: Label,
    timer: Timer,
}

impl Component for Clock {
    fn destroy(&self) {
        self.timer.remove();
        self.label.destroy();
    }
}

impl Clock {
    pub fn init(config: ConfigGroup, bar: &mut Bar, container: &gtk::Box) {
        let label = Label::new(None);
        super::init_widget(&label, &config, bar, container);
        label.show();

        // get config
        let symbols = SymbolFmt::new(config.get_str_or("format", "{timestamp}"));
        let timestamp = config
            .get_str_or("timestamp", "%Y-%m-%d %H:%M:%S")
            .to_string();
        let interval = config.get_int_or("interval", 1).max(1);

        // start timer
        let tick = clone!(label move || {
            let time = &format!("{}", Local::now().format(&timestamp));
            label.set_markup(&symbols.format(|sym| match sym {
                "timestamp" => time.to_string(),
                _ => sym.to_string(),
            }));
            gtk::Continue(true)
        });
        let timer = Timer::add_seconds(interval as u32, tick);

        bar.add_component(Box::new(Clock {
            label,
            timer,
        }));
    }
}
