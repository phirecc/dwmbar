use crate::module::{Module, ModuleResult};
use std::collections::HashMap;
use std::fs;
#[derive(Debug)]
pub struct MemoryUsage {}
impl Module for MemoryUsage {
    fn eval(&self) -> ModuleResult {
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
}
