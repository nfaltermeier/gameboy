use crate::memory::MemoryController;

pub enum WatchType {
    Rising,
    Constant,
    Falling,
}

pub struct Watch {
    name: &'static str,
    eval_fn: Box<dyn Fn(&dyn MemoryController) -> bool>,
    watch_type: WatchType,
    last_result: bool,
}

impl Watch {
    pub fn new(
        name: &'static str,
        eval_fn: Box<dyn Fn(&dyn MemoryController) -> bool>,
        watch_type: WatchType,
    ) -> Self {
        Watch {
            name,
            eval_fn,
            watch_type,
            last_result: false,
        }
    }

    pub fn test(&mut self, mem: &dyn MemoryController) -> bool {
        let val = (self.eval_fn)(mem);
        let trigger = match self.watch_type {
            WatchType::Rising => val && !self.last_result,
            WatchType::Constant => val,
            WatchType::Falling => !val && self.last_result,
        };

        self.last_result = val;
        trigger
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
