use crate::components::{Component, ComponentParams};
use gtk::prelude::*;
use gtk::Label;
use std::io::Error;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::util::{SymbolFmt, Timer};

pub struct Script {
    label: Label,
    timer: Timer,
    tx_term: mpsc::Sender<()>,
}

impl Component for Script {
    fn destroy(&self) {
        self.timer.remove();
        self.tx_term.send(()).ok();
        self.label.destroy();
    }
}

impl Script {
    pub fn init(params: ComponentParams) {
        let ComponentParams { config, window, container, .. } = params;
        if let Some(src) = config.get_string("src") {
            let (tx, rx) = mpsc::channel();
            let (tx_term, rx_term) = mpsc::channel();

            let interval = config.get_int_or("interval", 3).max(1);
            let symbols = SymbolFmt::new(config.get_str_or("format", "{stdout}"));

            if let Ok(output) = get_output(&src) {
                tx.send(output).ok();
            }

            thread::spawn(clone!((src, interval) move || {
                loop {
                    thread::sleep(Duration::from_secs(interval as u64));
                    if let Ok(output) = get_output(&src) {
                        tx.send(output).ok();
                    }
                    if let Ok(_) = rx_term.try_recv() {
                        break;
                    }
                }
            }));

            let label = Label::new(None);
            super::init_widget(&label, &config, &window, container);
            label.show();

            let tick = clone!(label move || {
                if let Ok((ref stdout, ref stderr, code)) = rx.try_recv() {
                    label.set_markup(&symbols.format(|sym| {
                        match sym {
                            "stdout" => stdout.to_string(),
                            "stderr" => stderr.to_string(),
                            "code" => code.to_string(),
                            _ => sym.to_string(),
                        }
                    }));
                }
                gtk::Continue(true)
            });
            let timer = Timer::add_seconds(interval as u32, tick);

            window.add_component(Box::new(Script {
                label,
                timer,
                tx_term,
            }));
        } else {
            warn!("src property missing from #{}", config.name);
        }
    }
}

fn get_output(src: &str) -> Result<(String, String, i32), Error> {
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(&src)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(0);
    Ok((stdout, stderr, code))
}
