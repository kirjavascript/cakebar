pub mod atom;
pub mod xcb;
pub mod gtk;
pub mod bsp;
pub mod i3;
pub mod ipc;
pub mod events;
pub mod workspace;

use self::events::{Event, EventValue, EventEmitter};
use self::workspace::Workspace;

use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum WMType {
    I3,
    Bsp,
    Unknown,
}

impl fmt::Display for WMType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", &self).to_lowercase())
    }
}

pub struct WMUtil(Rc<RefCell<Data>>);

struct Data {
    wm_type: WMType,
    events: EventEmitter<Event, EventValue>,
}

impl Clone for WMUtil {
    fn clone(&self) -> Self {
        WMUtil(self.0.clone())
    }
}

impl WMUtil {

    pub fn new() -> Self {
        let wm_type = if let Ok(_) = i3::connect() {
            WMType::I3
        } else if let Ok(_) = bsp::connect() {
            WMType::Bsp
        } else {
            WMType::Unknown
        };

        if wm_type != WMType::Unknown {
            info!("detected {}wm", wm_type);
        }

        let events = EventEmitter::new();

        let data = Rc::new(RefCell::new(Data {
            wm_type,
            events,
        }));

        let util = WMUtil(data);

        xcb::listen(&util);

        match util.get_wm_type() {
            WMType::I3 => {
                i3::listen(&util);
            },
            WMType::Bsp => {
                bsp::listen(&util);
            },
            _ => {},
        }

        util
    }

    // getters

    pub fn get_wm_type(&self) -> WMType {
        self.0.borrow().wm_type.clone()
    }

    // events

    pub fn add_listener<F: 'static>(&self, event: Event, callback: F)
        where F: Fn(Option<EventValue>) {
        self.0.borrow_mut().events.add_listener(event, callback);
    }

    #[allow(dead_code)]
    pub fn emit(&self, event: Event) {
        self.0.borrow().events.emit(event);
    }

    pub fn emit_value(&self, event: Event, value: EventValue) {
        self.0.borrow().events.emit_value(event, value);
    }

    // wm actions

    pub fn get_workspaces(&self) -> Option<Vec<Workspace>> {
        match self.0.borrow().wm_type {
            WMType::I3 => {
                match i3::connect() {
                    Ok(mut connection) => {
                        Some(i3::get_workspaces(&mut connection))
                    },
                    Err(_) => None
                }
            },
            WMType::Bsp => {
                match bsp::connect() {
                    Ok(mut connection) => {
                        Some(bsp::get_workspaces(&mut connection))
                    },
                    Err(_) => None
                }
            },
            _ => None
        }
    }

    pub fn focus_workspace(&self, workspace_name: &String) {
        match self.0.borrow().wm_type {
            WMType::I3 => {
                let command = format!("workspace {}", workspace_name);
                i3::run_command(&command);
            },
            WMType::Bsp => {
                let command = format!("desktop -f {}", workspace_name);
                bsp::run_command(command).ok();
            },
            _ => {},
        }
    }

    pub fn cycle_workspace(&self, forward: bool, monitor_index: i32) {
        match self.0.borrow().wm_type {
            WMType::I3 => {
                i3::cycle_workspace(forward, monitor_index);
            },
            WMType::Bsp => {
                bsp::cycle_workspace(forward, monitor_index);
            },
            _ => {},
        }
    }

    pub fn set_padding(&self, is_top: bool, padding: i32) {
        match self.0.borrow().wm_type {
            WMType::Bsp => {
                bsp::set_padding(is_top, padding);
            },
            // don't need to do this in i3
            _ => {},
        }
    }

}
