use crate::module::{Module, ModuleResult};
use std::fs;
#[derive(Debug)]
pub struct CpuTemperature {}
impl Module for CpuTemperature {
    fn eval(&self) -> ModuleResult {
        // TODO This currently would print an ugly "No such file or directory" error. I think using
        // `anyhow` this could be cleaned up with context on what file doesn't exist.
        // TODO The initial finding of the input file should be done once in a new() method. In
        // order for this to work the calling code in main.rs should handle this.
        // Maybe instead I could also store a result in the struct and just keep returning the
        // error in eval?
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
}
