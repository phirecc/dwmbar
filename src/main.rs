use std::{
    io::BufRead,
    io::BufReader,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
mod module;
mod modules;

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(screen: usize) -> usize;
    fn XStoreName(display: usize, window: usize, name: *const u8) -> i32;
    fn XDefaultRootWindow(display: usize) -> usize;
    fn XFlush(display: usize) -> i32;
}

fn watch_pulse_events(bar: Arc<Bar>) {
    let vol_idx = 3; // TODO: Maybe find this dynamically?
    let out = Command::new("pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let r = BufReader::new(out.stdout.unwrap());
    let mut count = 0;
    for line in r.lines() {
        let line = line.unwrap();
        if line.contains("change' on source") {
            count += 1;
            // We always get 2 such messages per volume change
            if count % 2 == 0 {
                bar.results.lock().unwrap()[vol_idx] =
                    (bar.parts[vol_idx].0).eval().unwrap_or_else(|_| Some("ERROR".to_string()));
                bar.display_results();
            }
        }
    }
}

struct Bar {
    parts: Vec<(Box<dyn module::Module>, usize)>,
    display: Mutex<usize>,
    root: usize,
    results: Mutex<Vec<Option<String>>>,
}

impl Bar {
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(0) };
        let root = unsafe { XDefaultRootWindow(display) };
        let parts: Vec<(Box<dyn module::Module>, usize)> = vec![
            (Box::new(modules::Vpn{}), 2),
            (Box::new(modules::Combined::new(Box::new(modules::CpuTemperature{}), Box::new(modules::LoadAvg{}))), 2),
            (Box::new(modules::MemoryUsage{}), 2),
            (Box::new(modules::Volume{}), 5),
            (Box::new(modules::Battery{}), 10),
            (Box::new(modules::DateTime{}), 10),
        ];
        Self {
            results: Mutex::new(vec![None; parts.len()]),
            parts,
            display: Mutex::new(display),
            root,
        }
    }

    fn setroot(&self, mut s: String) {
        s.push('\0'); // Because rust strings aren't zero-terminated
        let display = self.display.lock().unwrap();
        unsafe {
            XStoreName(*display, self.root, s.as_ptr());
            XFlush(*display);
        };
    }

    pub fn display_results(&self) {
        let s = {
            let r = self.results.lock().unwrap();
            let y: Vec<&str> = r.iter().flatten().map(|x| x.as_ref()).collect();
            y.join(" | ")
        };
        self.setroot(s);
    }

    pub fn run(&self) {
        let mut counter = 0;
        loop {
            {
                let mut r = self.results.lock().unwrap();
                for (i, part) in self.parts.iter().enumerate() {
                    if counter % part.1 == 0 {
                        let res = (part.0).eval();
                        r[i] = match res {
                            Ok(x) => x,
                            Err(e) => {
                                eprintln!("Module ({:?}) errored: {}", i, e);
                                Some("ERROR".into())
                            }
                        };
                    }
                }
            }
            self.display_results();
            counter += 1;
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn main() {
    let bar = Arc::new(Bar::new());
    let cl = bar.clone();
    thread::spawn(|| watch_pulse_events(cl));
    bar.run();
}
