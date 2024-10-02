use crate::memory::MemoryController;

pub const DEBUG_PRINT_PC: bool = false;
pub const DEBUG_PRINT_PPU: bool = false;
pub const DEBUG_PRINT_FRAME_TIME: bool = false;
pub const DEBUG_PRINT_VRAM_WRITES: bool = false;
pub const DEBUG_PRINT_INTERRUPTS: bool = false;

pub const DEBUG_PRINT_WHEN_PC: u16 = 0x28;
pub const DEBUG_PRINT_WHEN_PC_TIMES: u8 = 10;

pub enum WatchType {
    Rising,
    Constant,
    Falling,
}

pub struct WatchValue<T>
where
    T: PartialEq,
{
    pub name: &'static str,
    target_val: T,
    last_val: Option<T>,
    eval_fn: fn(&dyn MemoryController) -> T,
    watch_type: WatchType,
}

impl<T> WatchValue<T>
where
    T: PartialEq,
{
    pub fn new(
        name: &'static str,
        target_val: T,
        eval_fn: fn(&dyn MemoryController) -> T,
        watch_type: WatchType,
    ) -> Self {
        WatchValue {
            name,
            target_val,
            last_val: None,
            eval_fn,
            watch_type,
        }
    }

    pub fn test(&mut self, mem: &dyn MemoryController) -> bool {
        let val = (self.eval_fn)(mem);
        let trigger = match self.watch_type {
            WatchType::Rising => {
                let last_triggers = match &self.last_val {
                    None => true,
                    Some(lv) => *lv != val,
                };
                val == self.target_val && last_triggers
            }
            WatchType::Constant => val == self.target_val,
            WatchType::Falling => {
                let last_triggers = match &self.last_val {
                    None => false,
                    Some(lv) => *lv == val,
                };
                val != self.target_val && last_triggers
            }
        };

        self.last_val = Some(val);
        trigger
    }
}
