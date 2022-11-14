use std::{sync::mpsc, sync::Arc, sync::Mutex, thread, time::Duration};
mod module;
mod modules;

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(screen: usize) -> usize;
    fn XStoreName(display: usize, window: usize, name: *const u8) -> i32;
    fn XDefaultRootWindow(display: usize) -> usize;
    fn XFlush(display: usize) -> i32;
}

type BarPart = (Arc<dyn module::Module>, usize);

struct Bar {
    parts: Vec<BarPart>,
    display: Mutex<usize>,
    root: usize,
    results: Mutex<Vec<Option<String>>>,
}

impl Bar {
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(0) };
        let root = unsafe { XDefaultRootWindow(display) };
        let parts: Vec<BarPart> = vec![
            (Arc::new(modules::Vpn {}), 2),
            (
                Arc::new(modules::Combined::new(
                    Box::new(modules::CpuTemperature {}),
                    Box::new(modules::LoadAvg {}),
                )),
                2,
            ),
            (Arc::new(modules::MemoryUsage {}), 2),
            (Arc::new(modules::Volume {}), 5),
            (Arc::new(modules::Battery {}), 10),
            (Arc::new(modules::DateTime {}), 10),
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

    fn evaluate(part: &BarPart) -> Option<String> {
        let res = (part.0).eval();
        match res {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Module {:?} errored: {}", part, e);
                Some("ERROR".into())
            }
        }
    }
    pub fn start_watches(&self) {
        let (tx, rx) = mpsc::channel();
        // XXX This spawns a thread for every bar module. Most of them will not have watches and
        // exit immediately. This unneccessary thread startup overhead could be avoided.
        for (i, p) in self.parts.iter().enumerate() {
            let x = Arc::clone(&p.0);
            let tx = tx.clone();
            thread::spawn(move || x.watch(i, tx));
        }
        for r in rx {
            self.results.lock().unwrap()[r] = Self::evaluate(&self.parts[r]);
            self.display_results();
        }
    }
    pub fn run(&self) {
        let mut counter = 0;
        loop {
            {
                let mut r = self.results.lock().unwrap();
                for (i, part) in self.parts.iter().enumerate() {
                    if counter % part.1 == 0 {
                        r[i] = Self::evaluate(part);
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
    thread::spawn(move || cl.start_watches());
    bar.run();
}
