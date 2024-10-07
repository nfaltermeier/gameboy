use core::cmp::Reverse;

use crate::my_lib::sparse_vec::SparseVec;

pub struct DebugMetrics {
    jumps: SparseVec<u16, JumpData>,
}

impl DebugMetrics {
    pub fn new() -> Self {
        DebugMetrics {
            jumps: SparseVec::new(),
        }
    }

    pub fn jump_conditional(&mut self, source: u16, dest: u16, instruction_name: &'static str, condition_name: &'static str, jump_taken: bool) {
        match self.jumps.get_mut(&source) {
            Some(jump) => {
                if jump_taken {
                    jump.jump_taken = true;
                } else {
                    jump.jump_skipped = true;
                }
            },
            None => {
                self.jumps.insert(source, JumpData::new_conditional(source, dest, instruction_name, condition_name, jump_taken));
            }
        };
    }

    pub fn jump_not_conditional(&mut self, source: u16, dest: u16, instruction_name: &'static str) {
        if !self.jumps.contains_key(&source) {
            self.jumps.insert(source, JumpData::new_not_conditional(source, dest, instruction_name));
        }
    }

    pub fn print_jumps(&mut self, highlight_addrs: &Vec<u16>) {
        println!("Jumps:");

        self.jumps.cache_keys();
        for j in self.jumps.iter_keys_ordered().unwrap() {
            let entrypoint = *j >= 0x100;

            // todo: detect when interrupt ends
            // assuming all interrupy handlers immediately jump
            // if entrypoint || *j == 0x40 || *j == 0x48 || *j == 0x50 || *j == 0x58 || *j == 0x60 {
            if entrypoint {
                print!("{}", self.calc_jumps_tree(0, *j, &Vec::new(), highlight_addrs).1);

                if entrypoint {
                    break;
                }
            }
        }
    }

    const INDENT_STR: &'static str = "  ";

    fn calc_jumps_tree(&self, indent: usize, addr: u16, parent_conditionals: &Vec<u16>, highlight_addrs: &Vec<u16>) -> (usize, String) {
        let mut cur_jump_o = self.jumps.get_or_next(&addr).unwrap();
        let mut lines: usize = 0;
        let mut result = String::new();
        let mut last_addr = addr;

        while cur_jump_o.is_some() {
            let cur_jump = cur_jump_o.unwrap();
            let already_visited = cur_jump.conditional && parent_conditionals.contains(&cur_jump.source);

            if !already_visited {
                for addr in highlight_addrs.iter().filter(|addr| last_addr <= **addr && cur_jump.source > **addr) {
                    result.push_str(format!("{}---- {addr:#06x} ----\n", Self::INDENT_STR.repeat(indent)).as_str());
                    lines += 1;
                }
            }

            let next: u16;
            if cur_jump.conditional && cur_jump.jump_skipped && cur_jump.jump_taken {
                if !already_visited {
                    let mut new_parents = parent_conditionals.clone();
                    new_parents.push(cur_jump.source);
                    let taken = self.calc_jumps_tree(indent + 1, cur_jump.dest, &new_parents, highlight_addrs);
                    let skipped = self.calc_jumps_tree(indent + 1, cur_jump.source + 1, &new_parents, highlight_addrs);

                    result.push_str(format!("{}{} -> {:#06x}\n", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                    if skipped.0 <= taken.0 {
                        result.push_str(format!("{}skipped:\n{}", Self::INDENT_STR.repeat(indent), skipped.1).as_str());
                        result.push_str(format!("{}taken:\n{}", Self::INDENT_STR.repeat(indent), taken.1).as_str());
                    } else {
                        result.push_str(format!("{}taken:\n{}", Self::INDENT_STR.repeat(indent), taken.1).as_str());
                        result.push_str(format!("{}skipped:\n{}", Self::INDENT_STR.repeat(indent), skipped.1).as_str());
                    }
                    lines += skipped.0 + taken.0 + 3;
                } else {
                    result.push_str(format!("{}{} -> {:#06x} (previous conditional)\n", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                    lines += 1;
                }

                return (lines, result);
            } else if !cur_jump.conditional {
                result.push_str(Self::INDENT_STR.repeat(indent).as_str());
                result.push_str(cur_jump.name.as_str());
                result.push('\n');
                lines += 1;
                next = cur_jump.dest;
            } else if cur_jump.jump_skipped {
                result.push_str(format!("{}{} -> {:#06x} (skipped)\n", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.source + 1).as_str());
                lines += 1;
                next = cur_jump.source + 1;
            } else {
                result.push_str(format!("{}{} -> {:#06x} (taken)\n", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                lines += 1;
                next = cur_jump.dest;
            }

            // todo: branching from jp (hl) instructions with destinations targets
            // todo: prevent infinite loops? maybe already done.

            if already_visited {
                return (lines, result);
            }

            last_addr = next;
            cur_jump_o = self.jumps.get_or_next(&next).unwrap();
        }

        (lines, result)
    }

    pub fn clear_jumps(&mut self) {
        self.jumps.clear();
    }
}

pub struct JumpData {
    source: u16,
    dest: u16,
    name: String,
    conditional: bool,
    jump_taken: bool,
    jump_skipped: bool,
}

impl JumpData {
    pub fn new_conditional(source: u16, dest: u16, instruction_name: &'static str, condition_name: &'static str, jump_taken: bool) -> Self {
        JumpData {
            source,
            dest,
            name: format!("{source:#06x}: {instruction_name} {condition_name}"),
            conditional: true,
            jump_taken,
            jump_skipped: !jump_taken,
        }
    }

    pub fn new_not_conditional(source: u16, dest: u16, instruction_name: &'static str) -> Self {
        JumpData {
            source,
            dest,
            name: format!("{source:#06x}: {instruction_name} -> {dest:#06x}"),
            conditional: false,
            jump_taken: true,
            jump_skipped: false,
        }
    }
}
