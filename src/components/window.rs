use super::{Component, Bar, gtk, ComponentConfig};
use gtk::prelude::*;
use gtk::{Label};

use wm::events::{Event, EventValue};

pub struct Window { }

impl Component for Window {
    fn init(container: &gtk::Box, config: &ComponentConfig, bar: &Bar){
        let label = Label::new(None);

        Self::init_widget(&label, container, config, bar);
        label.show();
        let trunc = config.get_int_or("truncate", 100) as usize;

        bar.wm_util.add_listener(Event::Window, clone!(label
            move |event_opt| {
                if let Some(EventValue::String(name)) = event_opt {
                    let name = if name.chars().count() > trunc {
                        let name = name
                            .char_indices()
                            .filter(|x| x.0 <= trunc)
                            .fold("".to_string(), |acc, cur| {
                                acc + &cur.1.to_string()
                            });
                        format!("{}…", name)
                    } else {
                        name
                    };
                    label.set_text(&name);
                }
            }
        ));
    }
}