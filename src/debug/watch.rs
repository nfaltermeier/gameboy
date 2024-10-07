use std::fmt::Display;

use crate::memory::MemoryController;

pub trait Watch {
    fn test(&mut self, mem: &dyn MemoryController) -> bool;
    fn name(&self) -> String;
}

pub enum WatchFnType {
    Rising,
    Constant,
    Falling,
}

pub struct WatchFn {
    name: &'static str,
    eval_fn: Box<dyn Fn(&dyn MemoryController) -> bool>,
    watch_type: WatchFnType,
    last_result: bool,
}

impl WatchFn {
    pub fn new(
        name: &'static str,
        eval_fn: Box<dyn Fn(&dyn MemoryController) -> bool>,
        watch_type: WatchFnType,
    ) -> Self {
        WatchFn {
            name,
            eval_fn,
            watch_type,
            last_result: false,
        }
    }
}

impl Watch for WatchFn {
    fn test(&mut self, mem: &dyn MemoryController) -> bool {
        let val = (self.eval_fn)(mem);
        let trigger = match self.watch_type {
            WatchFnType::Rising => val && !self.last_result,
            WatchFnType::Constant => val,
            WatchFnType::Falling => !val && self.last_result,
        };

        self.last_result = val;
        trigger
    }

    fn name(&self) -> String {
        String::from(self.name)
    }
}

pub struct WatchValueChange<T> where T: PartialEq + Display {
    name: &'static str,
    eval_fn: Box<dyn Fn(&dyn MemoryController) -> T>,
    last_value: Option<T>
}

impl<T> WatchValueChange<T> where T: PartialEq + Display {
    pub fn new(
        name: &'static str,
        eval_fn: Box<dyn Fn(&dyn MemoryController) -> T>
    ) -> Self {
        WatchValueChange {
            name,
            eval_fn,
            last_value: None,
        }
    }
}

impl<T> Watch for WatchValueChange<T> where T: PartialEq + Display {
    fn test(&mut self, mem: &dyn MemoryController) -> bool {
        let val = (self.eval_fn)(mem);
        let trigger = match &self.last_value {
            None => false,
            Some(lv) => val != *lv
        };

        self.last_value = Some(val);
        trigger
    }

    fn name(&self) -> String {
        match &self.last_value {
            None => format!("{} value: (No value recorded)", self.name),
            Some(v) => format!("{} value: {}", self.name, v),
        }
    }
}
