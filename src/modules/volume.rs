use crate::module::{Module, ModuleResult};
use std::{
    io::BufRead,
    io::BufReader,
    process::{Command, Stdio},
};
#[derive(Debug)]
pub struct Volume {}
impl Module for Volume {
    // TODO: Maybe look into implemeting this natively with a pulseaudio or pipewire library
    fn eval(&self) -> ModuleResult {
        let out = Command::new("pamixer").arg("--get-volume").output()?;
        Ok(Some(String::from_utf8(out.stdout)?.trim().into()))
    }
    fn watch(&self, idx: usize, tx: std::sync::mpsc::Sender<usize>) {
        let out = Command::new("pactl")
            .arg("subscribe")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let r = BufReader::new(out.stdout.unwrap());
        for line in r.lines() {
            let line = line.unwrap();
            if line.contains("change' on sink") {
                if let Err(e) = tx.send(idx) {
                    eprintln!("tx failed in watch {}: {}", idx, e)
                }
            }
        }
    }
}
