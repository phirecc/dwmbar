use crate::module::{Module, ModuleResult};
use std::process::Command;
pub struct Volume {
}
impl Module for Volume {
    // TODO: Maybe look into implemeting this natively with a pulseaudio or pipewire library
    fn eval(&self) -> ModuleResult {
        let out = Command::new("pamixer").arg("--get-volume").output()?;
        Ok(Some(String::from_utf8(out.stdout)?.trim().into()))
    }
}
