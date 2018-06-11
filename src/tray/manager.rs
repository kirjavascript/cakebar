use super::atom;
use xcb;

const CLIENT_MESSAGE: u8 = xcb::CLIENT_MESSAGE | 0x80; // 0x80 flag for client messages

const WM_NAME: &'static str = "System Tray";
const SYSTEM_TRAY_REQUEST_DOCK: u32 = 0;
const SYSTEM_TRAY_BEGIN_MESSAGE: u32 = 1;
const SYSTEM_TRAY_CANCEL_MESSAGE: u32 = 2;

pub struct Manager<'a> {
    conn: &'a xcb::Connection,
    atoms: &'a atom::Atoms<'a>,
    screen: usize,
    icon_size: u16,
    bg: u32,
    window: xcb::Window,
    children: Vec<xcb::Window>,
    timestamp: xcb::Timestamp,
    finishing: bool,
}

impl<'a> Manager<'a> {
    pub fn new<'b>(
        conn: &'b xcb::Connection,
        atoms: &'b atom::Atoms,
        screen: usize,
        icon_size: u16,
        bg: u32
    ) -> Manager<'b> {
        Manager::<'b> {
            conn: conn,
            atoms: atoms,
            screen: screen,
            icon_size: icon_size,
            bg: bg,
            window: conn.generate_id(),
            children: vec![],
            timestamp: 0,
            finishing: false,
        }
    }

    pub fn create(&self) {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen).unwrap();

        xcb::create_window(
            &self.conn,
            xcb::COPY_FROM_PARENT as u8,
            self.window,
            screen.root(),
            0, 0,
            self.icon_size, self.icon_size,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[
                (xcb::CW_BACK_PIXEL, self.bg),
                (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE),
                (xcb::CW_OVERRIDE_REDIRECT, 1),
            ]
        );
        self.set_property(
            xcb::ATOM_WM_NAME,
            xcb::ATOM_STRING,
            8,
            WM_NAME.as_bytes()
        );
        self.set_property(
            xcb::ATOM_WM_CLASS,
            xcb::ATOM_STRING,
            8,
            format!("{0}\0{0}", ::NAME).as_bytes()
        );
        self.set_property(
            self.atoms.get(atom::_NET_SYSTEM_TRAY_ORIENTATION),
            xcb::ATOM_CARDINAL,
            32,
            &[0 as u32] // 0 is horizontal, 1 is vertical
        );

        // set window type to utility (so it floats)
        self.set_property(
            self.atoms.get(atom::_NET_WM_WINDOW_TYPE),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::_NET_WM_WINDOW_TYPE_DOCK)]
        );
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_APPEND as u8,
            self.window,
            self.atoms.get(atom::_NET_WM_WINDOW_TYPE),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::_NET_WM_WINDOW_TYPE_NORMAL)]
        );

        // ??? (seems set in polybar)
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            self.atoms.get(atom::_NET_SYSTEM_TRAY_VISUAL),
            xcb::ATOM_VISUALID,
            32,
            &[screen.root_visual()]
        );

        // skip showing in taskbar
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            self.atoms.get(atom::_NET_WM_STATE),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::_NET_WM_STATE_SKIP_TASKBAR)]
        );

        // seems to stop delete event ruining our fun
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            self.atoms.get(atom::WM_PROTOCOLS),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::WM_DELETE_WINDOW)]
        );

        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_APPEND as u8,
            self.window,
            self.atoms.get(atom::WM_PROTOCOLS),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::WM_TAKE_FOCUS)]
        );

        // keeps tray on every workspace screen
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_APPEND as u8,
            self.window,
            self.atoms.get(atom::_NET_WM_STATE),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::_NET_WM_STATE_STICKY)]
        );

        // make decorationless
        xcb::change_property_checked(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            self.atoms.get(atom::_MOTIF_WM_HINTS),
            xcb::ATOM_INTEGER,
            32,
            &[0b10, 0, 0, 0, 0]
        );

        // disable compton shadow (apparently)
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            self.atoms.get(atom::_COMPTON_SHADOW),
            xcb::ATOM_CARDINAL,
            32,
            &[0]
        );

        // adds resize event
        // xcb::change_window_attributes(self.conn, self.window, &[
        //     (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY),
        // ]);
        //

        // restack window (fixes always on top bug)
        // if let Ok(reply) = xcb::query_tree(self.conn, screen.root()).get_reply() {
        //     let children = reply.children();
        //     let value_mask = (xcb::CONFIG_WINDOW_STACK_MODE | xcb::CONFIG_WINDOW_SIBLING) as u16;
        //     // find the first i3 window
        //     for child in children {
        //         let wm_name = xcb_get_wm_name(self.conn, *child);
        //         if wm_name.contains("i3") {
        //             // put the window directly above it
        //             xcb::configure_window_checked(self.conn, self.window, &[
        //                 (value_mask, *child),
        //                 (value_mask, xcb::STACK_MODE_ABOVE),
        //             ]);
        //             break;
        //         }
        //     }
        // }

        // attempt to stack gtk bar BELOW tray
        // let value_mask = (xcb::CONFIG_WINDOW_STACK_MODE | xcb::CONFIG_WINDOW_SIBLING) as u16;

        // if let Ok(reply) = xcb::query_tree(&self.conn, screen.root()).get_reply() {
        //     let i3_opt = reply.children().iter().find(|child| {
        //          xcb_get_wm_name(&self.conn, **child).contains("i3")
        //     });
        //     let bar_opt = reply.children().iter().find(|child| {
        //         ::NAME == xcb_get_wm_name(&self.conn, **child)
        //     });
        //     if let Some(bar) = bar_opt {
        //         if let Some(i3) = i3_opt {
        //             println!("{:#?}", (i3, bar));

        //             xcb::configure_window_checked(&self.conn, *i3, &[
        //                 (value_mask, *bar),
        //                 (value_mask, xcb::STACK_MODE_ABOVE),
        //             ]);

        //             println!("{:#?}", "swapped");
        //             self.conn.flush();
        //         }
        //     }
        // }

        self.conn.flush();

        // debug window order
        // if let Ok(reply) = xcb::query_tree(&self.conn, screen.root()).get_reply() {
        //     for i in reply.children() {
        //         println!("{:#?} {}", xcb_get_wm_name(&self.conn, *i), i);
        //     }
        // }
    }

    pub fn set_property<T>(&self, name: xcb::Atom, type_: xcb::Atom, format: u8, data: &[T]) {
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            name,
            type_,
            format,
            data
        );
    }

    pub fn is_selection_available(&self) -> bool {
        let selection = self.atoms.get(atom::_NET_SYSTEM_TRAY_S0);
        let owner = xcb::get_selection_owner(self.conn, selection).get_reply().unwrap().owner();
        owner == xcb::NONE
    }

    pub fn take_selection(&mut self, timestamp: xcb::Timestamp) -> bool {
        let selection = self.atoms.get(atom::_NET_SYSTEM_TRAY_S0);
        xcb::set_selection_owner(self.conn, self.window, selection, timestamp);
        let owner = xcb::get_selection_owner(self.conn, selection).get_reply().unwrap().owner();
        let ok = owner == self.window;
        if ok {
            self.timestamp = timestamp;
            let setup = self.conn.get_setup();
            let screen = setup.roots().nth(self.screen).unwrap();

            let client_event = xcb::ClientMessageEvent::new(
                32, // 32 bits (refers to data)
                screen.root(),
                self.atoms.get(atom::MANAGER),
                xcb::ClientMessageData::from_data32([timestamp, selection, self.window, 0, 0])
            );
            xcb::send_event(self.conn, false, screen.root(), xcb::EVENT_MASK_STRUCTURE_NOTIFY, &client_event);
            self.conn.flush();
        }
        ok
    }

    pub fn adopt(&mut self, window: xcb::Window) {
        let offset = (self.children.len() as u16 * self.icon_size) as i16;
        xcb::change_window_attributes(self.conn, window, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY),
        ]);
        xcb::reparent_window(self.conn, window, self.window, offset, 0);
        xcb::map_window(self.conn, window);
        self.force_size(window, None);
        self.conn.flush();
        self.children.push(window);
        self.reposition();
    }

    pub fn forget(&mut self, window: xcb::Window) {
        self.children.retain(|child| *child != window);
        for (index, child) in self.children.iter().enumerate() {
            let window = *child;
            let xpos = index as u32 * self.icon_size as u32;
            xcb::configure_window(&self.conn, window, &[
                (xcb::CONFIG_WINDOW_X as u16, xpos)
            ]);
        }
        self.reposition();
    }

    pub fn force_size(&self, window: xcb::Window, dimensions: Option<(u16, u16)>) {
        let dimensions = dimensions.unwrap_or_else(|| {
            let geometry = xcb::get_geometry(self.conn, window).get_reply().unwrap();
            (geometry.width(), geometry.height())
        });
        if dimensions != (self.icon_size, self.icon_size) {
            xcb::configure_window(self.conn, window, &[
                (xcb::CONFIG_WINDOW_WIDTH as u16, self.icon_size as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, self.icon_size as u32)
            ]);
            self.conn.flush();
        }
    }

    pub fn reposition(&self) {
        let width = self.children.len() as u16 * self.icon_size;
        if width > 0 {
            // let setup = self.conn.get_setup();
            // let screen = setup.roots().nth(self.screen).unwrap();
                // &HorizontalAlign::Right => screen.width_in_pixels() - width

            let x = 140;
            let y = 3;
            xcb::configure_window(self.conn, self.window, &[
                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                (xcb::CONFIG_WINDOW_WIDTH as u16, width as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, 20),
            ]);
            xcb::map_window(self.conn, self.window);
        }
        else {
            xcb::unmap_window(self.conn, self.window);
        }
        self.conn.flush();
    }

    pub fn finish(&mut self) {
        self.finishing = true;
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen).unwrap();
        let root = screen.root();

        for child in self.children.iter() {
            let window = *child;
            xcb::change_window_attributes(self.conn, window, &[
                (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)
            ]);
            xcb::unmap_window(self.conn, window);
            xcb::reparent_window(self.conn, window, root, 0, 0);
        }
        xcb::change_window_attributes(self.conn, self.window, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY)
        ]);
        xcb::destroy_window(self.conn, self.window);
        self.conn.flush();
    }

    pub fn handle_event(&mut self, event: xcb::GenericEvent) -> Option<i32> {
        if self.finishing {
            self.handle_event_finishing(event)
        }
        else {
            self.handle_event_normal(event)
        }
    }

    fn handle_event_normal(&mut self, event: xcb::GenericEvent) -> Option<i32> {
            match event.response_type() {
                xcb::PROPERTY_NOTIFY if self.timestamp == 0 => {
                    let event: &xcb::PropertyNotifyEvent = unsafe {
                        xcb::cast_event(&event)
                    };
                    if !self.take_selection(event.time()) {
                        println!("Could not take ownership of tray selection. Maybe another tray is also running?");
                        return Some(super::EXIT_FAILED_SELECT)
                    }
                },
                CLIENT_MESSAGE => {
                    let event: &xcb::ClientMessageEvent = unsafe {
                        xcb::cast_event(&event)
                    };
                    if event.type_() == self.atoms.get(atom::_NET_SYSTEM_TRAY_OPCODE) {
                        let data = event.data().data32();
                        let opcode = data[1];
                        let window = data[2];
                        match opcode {
                            SYSTEM_TRAY_REQUEST_DOCK => {
                                self.adopt(window);
                            },
                            SYSTEM_TRAY_BEGIN_MESSAGE => {},
                            SYSTEM_TRAY_CANCEL_MESSAGE => {},
                            _ => { unreachable!("Invalid opcode") }
                        }
                    }
                },
                xcb::REPARENT_NOTIFY => {
                    let event: &xcb::ReparentNotifyEvent = unsafe {
                        xcb::cast_event(&event)
                    };
                    if event.parent() != self.window {
                        self.forget(event.window());
                    }
                },
                xcb::DESTROY_NOTIFY => {
                    let event: &xcb::DestroyNotifyEvent = unsafe {
                        xcb::cast_event(&event)
                    };
                    self.forget(event.window());
                },
                xcb::CONFIGURE_NOTIFY => {
                    let event: &xcb::ConfigureNotifyEvent = unsafe {
                        xcb::cast_event(&event)
                    };
                    self.force_size(event.window(), Some((event.width(), event.height())));
                },
                xcb::SELECTION_CLEAR => {
                    self.finish();
                },
                // // resize
                // 150 => {
                //     // force window in place
                //     xcb::configure_window(self.conn, self.window, &[
                //         (xcb::CONFIG_WINDOW_X as u16, 1440),
                //         (xcb::CONFIG_WINDOW_Y as u16, 1056),
                //     ]);
                //     self.conn.flush();
                // },
                _ => {
                }
            }
            None
    }

    fn handle_event_finishing(&mut self, event: xcb::GenericEvent) -> Option<i32> {
        if event.response_type() == xcb::DESTROY_NOTIFY {
            let event: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&event) };
            if event.window() == self.window {
                return Some(0)
            }
        }
        None
    }
}

// fn xcb_get_wm_name(conn: &xcb::Connection, id: u32) -> String {
//     let window: xcb::Window = id;
//     let long_length: u32 = 8;
//     let mut long_offset: u32 = 0;
//     let mut buf = Vec::new();
//     loop {
//         let cookie = xcb::get_property(
//             &conn,
//             false,
//             window,
//             xcb::ATOM_WM_NAME,
//             xcb::ATOM_STRING,
//             long_offset,
//             long_length,
//         );
//         match cookie.get_reply() {
//             Ok(reply) => {
//                 let value: &[u8] = reply.value();
//                 buf.extend_from_slice(value);
//                 match reply.bytes_after() {
//                     0 => break,
//                     _ => {
//                         let len = reply.value_len();
//                         long_offset += len / 4;
//                     }
//                 }
//             }
//             Err(err) => {
//                 println!("{:?}", err);
//                 break;
//             }
//         }
//     }
//     let result = String::from_utf8(buf).unwrap();
//     let results: Vec<&str> = result.split('\0').collect();
//     results[0].to_string()
// }
