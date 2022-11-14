use std::{error::Error, sync::mpsc};
pub type ModuleResult = Result<Option<String>, Box<dyn Error>>;

pub trait Module: Sync + Send + std::fmt::Debug {
    fn eval(&self) -> ModuleResult;
    fn watch(&self, _idx: usize, _tx: mpsc::Sender<usize>) {}
}
