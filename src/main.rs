use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::BufRead,
    io::BufReader,
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(screen: usize) -> usize;
    fn XStoreName(display: usize, window: usize, name: *const u8) -> i32;
    fn XDefaultRootWindow(display: usize) -> usize;
    fn XFlush(display: usize) -> i32;
}

type ModuleResult = Result<Option<String>, Box<dyn Error>>;

fn datetime() -> ModuleResult {
    let dt = chrono::Local::now();
    Ok(Some(dt.format("%d/%m/%y %a %H:%M").to_string()))
}

fn battery() -> ModuleResult {
    let base = "/sys/class/power_supply/BAT0/";
    let charge_now = fs::read_to_string(base.to_owned() + "charge_now")?;
    let charge_full = fs::read_to_string(base.to_owned() + "charge_full")?;
    let charge_now: f64 = charge_now.trim().parse()?;
    let charge_full: f64 = charge_full.trim().parse()?;
    let mut status = "";
    if fs::read_to_string(base.to_owned() + "status")?.trim() == "Charging" {
        status = "CHR ";
    }
    Ok(Some(format!(
        "{}{}%",
        status,
        ((charge_now as f64 / charge_full as f64) * 100.0) as usize
    )))
}

// TODO: Maybe look into implemeting this natively with a pulseaudio or pipewire library
fn volume() -> ModuleResult {
    let out = Command::new("pamixer").arg("--get-volume").output()?;
    Ok(Some(format!("{}", String::from_utf8(out.stdout)?.trim())))
}

fn loadavg() -> ModuleResult {
    let s = fs::read_to_string("/proc/loadavg")?;
    Ok(Some(
        s.split(" ")
            .next()
            .ok_or("loadavg missing first part")?
            .to_string(),
    ))
}

fn cputemp() -> ModuleResult {
    // TODO This currently would print an ugly "No such file or directory" error. I think using
    // `anyhow` this could be cleaned up with context on what file doesn't exist.
    let mut mon = fs::read_dir("/sys/devices/platform/coretemp.0/hwmon/")?
        .next()
        .ok_or("Missing cputemp hwmon")??
        .path();
    mon.push("temp1_input");
    // Unwrap: I don't think the OsString should ever fail to convert into a string.
    let t: usize = fs::read_to_string(mon.into_os_string().into_string().unwrap())?
        .trim()
        .parse()?;
    Ok(Some(format!("{}°C", (t / 1000))))
}

fn memusage() -> ModuleResult {
    let mut m = HashMap::new();
    let f = fs::read_to_string("/proc/meminfo")?;
    for line in f.lines() {
        let mut s = line.split_whitespace();
        m.insert(
            s.next().ok_or("Missing first part")?.trim_end_matches(':'),
            s.next().ok_or("Missing second part")?.parse::<usize>()?,
        );
    }
    // MemTotal - MemFree - Buffers - (Cached + SReclaimable - Shmem)
    let mem_used = m.get("MemTotal").ok_or("")?
        - m.get("MemFree").ok_or("")?
        - m.get("Buffers").ok_or("")?
        - (m.get("Cached").ok_or("")? + m.get("SReclaimable").ok_or("")?
            - m.get("Shmem").ok_or("")?);
    Ok(Some(format!(
        "{:.1}G/{:.1}G",
        mem_used as f64 / 1024.0 / 1024.0,
        *m.get("MemTotal").ok_or("")? as f64 / 1024.0 / 1024.0
    )))
}

fn vpn() -> ModuleResult {
    let vpns = ["wg0", "wgnord", "wg-mullvad"];
    for vpn in vpns {
        if Path::new(&("/sys/class/net/".to_owned() + vpn)).exists() {
            return Ok(Some("VPN".to_string()));
        }
    }
    Ok(None)
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
                    (bar.parts[vol_idx].0)().unwrap_or(Some("ERROR".to_string()));
                bar.display_results();
            }
        }
    }
}

struct Bar {
    parts: Vec<(Box<fn() -> ModuleResult>, usize)>,
    display: Mutex<usize>,
    root: usize,
    results: Mutex<Vec<Option<String>>>,
}

impl Bar {
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(0) };
        let root = unsafe { XDefaultRootWindow(display) };
        let parts: Vec<(Box<fn() -> ModuleResult>, usize)> = vec![
            (Box::new(vpn), 2),
            (
                Box::new(|| {
                    Ok(Some(format!(
                        "{} {}",
                        cputemp()?.unwrap(),
                        loadavg()?.unwrap()
                    )))
                }),
                2,
            ),
            (Box::new(memusage), 2),
            (Box::new(volume), 5),
            (Box::new(battery), 10),
            (Box::new(datetime), 10),
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
                    if counter % (*part).1 == 0 {
                        let res = (part.0)();
                        r[i] = match res {
                            Ok(x) => x,
                            Err(e) => {
                                eprintln!("Module {} ({:?}) errored: {}", i, part, e);
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
