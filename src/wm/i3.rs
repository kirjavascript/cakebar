use gtk;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::event::{Event as I3Event};
use wm::events::Event;

use std::thread;
use std::sync::mpsc;

use futures::Future;

enum I3Msg {
    Mode(String),
    Window(String),
    Workspace,
}

pub fn listen(wm_util: &::wm::WMUtil) {

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let listener_result = I3EventListener::connect();
        match listener_result {
            Ok(mut listener) => {
                let subs = [
                    Subscription::Mode,
                    Subscription::Window,
                    Subscription::Workspace,
                ];
                listener.subscribe(&subs).unwrap();

                for event in listener.listen() {
                    match event {
                        Ok(message) => {
                            match message {
                                I3Event::ModeEvent(e) => {
                                    tx.send(Ok(I3Msg::Mode(e.change)))
                                },
                                I3Event::WindowEvent(e) => {
                                    let name = e.container.name.unwrap_or("".to_string());
                                    tx.send(Ok(I3Msg::Window(name)))
                                },
                                I3Event::WorkspaceEvent(_e) => {
                                    // Focus Init Empty Urgent Rename Reload Restored Move Unknown
                                    tx.send(Ok(I3Msg::Workspace))
                                },
                                _ => unreachable!(),
                            };
                        },
                        Err(err) => {
                            // listener is rip
                            tx.send(Err(format!("{}", err))).unwrap();
                            break;
                        },
                    };
                }
            },
            Err(err) => {
                // socket failed to connect
                tx.send(Err(format!("{}", err))).unwrap();
            },
        };
    });

    gtk::timeout_add(10, clone!(wm_util move || {
        if let Ok(msg_result) = rx.try_recv() {
            match msg_result {
                Ok(msg) => {
                    match msg {
                        I3Msg::Mode(value) => {
                            wm_util.data
                                .borrow_mut()
                                .events
                                .emit_value(Event::Mode, value)
                                .wait()
                                .unwrap();
                        },
                        I3Msg::Window(value) => {
                            wm_util.data
                                .borrow_mut()
                                .events
                                .emit_value(Event::Window, value)
                                .wait()
                                .unwrap();
                        },
                        I3Msg::Workspace => {
                            wm_util.data
                                .borrow_mut()
                                .events
                                .emit(Event::Workspace)
                                .wait()
                                .unwrap();
                        },
                    }
                },
                Err(err) => {
                    info!("{}, restarting thread", err.to_lowercase());
                    gtk::timeout_add(100, move || {
                        // listen(&stream);
                        gtk::Continue(false)
                    });
                    return gtk::Continue(false);
                },
            };
        }
        gtk::Continue(true)
    }));
}
