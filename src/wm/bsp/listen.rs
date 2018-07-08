use gtk;

use std::thread;
use std::sync::mpsc;

// use std::os::unix::net::{UnixStream};
use std::io::{Read}; // Error, Write,

use wm::bsp;
use wm::events::{Event, EventValue};

pub fn listen(wm_util: &::wm::WMUtil) {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        match bsp::connect() {
            Ok(mut stream) => {
                bsp::write_message(&mut stream, "subscribe desktop monitor report".to_string()).ok();

                let mut current = [0; 1];
                let mut msg: Vec<u8> = Vec::new();
                loop {
                    if let Ok(_) = stream.read(&mut current) {
                        if current[0] == 10 {
                            tx.send(Ok(String::from_utf8(msg.clone()))).unwrap();
                            msg.clear();
                        } else {
                            msg.push(current[0]);
                        }
                    }
                }
            },
            Err(err) => {
                tx.send(Err(format!("{}", err))).unwrap();
            },
        }
    });

    gtk::timeout_add(10, clone!(wm_util move || {
        if let Ok(msg_result) = rx.try_recv() {
            match msg_result {
                Ok(msg) => {
                    if let Ok(msg) = msg {
                        if msg.starts_with("W") {
                            let workspaces = bsp::parse_workspaces(msg);
                            wm_util.emit_value(
                                Event::Workspace,
                                EventValue::Workspaces(workspaces),
                            );
                        }
                    }
                },
                Err(err) => {
                    warn!("{}, restarting thread", err.to_lowercase());
                    gtk::timeout_add(100, clone!(wm_util move || {
                        listen(&wm_util);
                        gtk::Continue(false)
                    }));
                    return gtk::Continue(false);
                },
            };
        }
        gtk::Continue(true)
    }));
}
