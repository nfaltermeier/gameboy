use std::collections::HashSet;

use crate::my_lib::sparse_vec::SparseVec;

pub struct DebugMetrics {
    jumps: SparseVec<u16, JumpData>,
}

struct MultipleDestinationsData {
    lines: usize,
    dest: u16,
    result: String,
}

struct JumpTreeData {
    lines: usize,
    result: String,
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
                assert_eq!(jump.dest, dest, "conditional jumps with multiple destinations (dynamic programming?) not supported. Disable tracking jumps to prevent this panic.");

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
        match self.jumps.get_mut(&source) {
            Some(jump) => {
                jump.update_destinations(dest);
            },
            None => {
                self.jumps.insert(source, JumpData::new_not_conditional(source, dest, instruction_name));
            }
        };
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
                print!("{}", self.calc_jumps_tree(0, *j, &mut Vec::new(), highlight_addrs).result);

                if entrypoint {
                    break;
                }
            }
        }
    }

    const INDENT_STR: &'static str = "  ";

    fn calc_jumps_tree(&self, indent: usize, addr: u16, mut parents: &mut Vec<u16>, highlight_addrs: &Vec<u16>) -> JumpTreeData {
        let mut cur_jump_o = self.jumps.get_or_next(&addr).unwrap();
        let mut lines: usize = 0;
        let mut result = String::new();
        let mut last_addr = addr;

        while cur_jump_o.is_some() {
            let cur_jump = cur_jump_o.unwrap();
            let already_visited = parents.contains(&cur_jump.source);
            parents.push(cur_jump.source);

            if !already_visited {
                for addr in highlight_addrs.iter().filter(|addr| last_addr <= **addr && cur_jump.source > **addr) {
                    result.push_str(format!("{}---- {addr:#06x} ----\n", Self::INDENT_STR.repeat(indent)).as_str());
                    lines += 1;
                }
            }

            let next: u16;
            if cur_jump.conditional {
                if cur_jump.jump_skipped && cur_jump.jump_taken {
                    if !already_visited {
                        let taken = self.calc_jumps_tree(indent + 1, cur_jump.dest, &mut parents, highlight_addrs);

                        if taken.lines == 1 && taken.result.trim_start().starts_with(format!("{:#06x}", cur_jump.source).as_str()) {
                            result.push_str(format!("{}{} -> {:#06x} (taken simple loop, following skipped)", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                            lines += 1;
                            next = cur_jump.source + 1;
                        } else {
                            let skipped = self.calc_jumps_tree(indent + 1, cur_jump.source + 1, &mut parents, highlight_addrs);
        
                            result.push_str(format!("{}{} -> {:#06x}\n", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                            if skipped.lines <= taken.lines {
                                result.push_str(format!("{}skipped:\n{}", Self::INDENT_STR.repeat(indent), skipped.result).as_str());
                                result.push_str(format!("{}taken:\n{}", Self::INDENT_STR.repeat(indent), taken.result).as_str());
                            } else {
                                result.push_str(format!("{}taken:\n{}", Self::INDENT_STR.repeat(indent), taken.result).as_str());
                                result.push_str(format!("{}skipped:\n{}", Self::INDENT_STR.repeat(indent), skipped.result).as_str());
                            }
                            lines += skipped.lines + taken.lines + 3;
                            return JumpTreeData { lines, result };
                        }
                    } else {
                        result.push_str(format!("{}{} -> {:#06x} (taken and skipped, already visited)\n", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                        lines += 1;
                        return JumpTreeData { lines, result };
                    }
                } else if cur_jump.jump_skipped {
                    result.push_str(format!("{}{} -> {:#06x} (skipped)", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.source + 1).as_str());
                    lines += 1;
                    next = cur_jump.source + 1;
                } else {
                    result.push_str(format!("{}{} -> {:#06x} (taken)", Self::INDENT_STR.repeat(indent), cur_jump.name, cur_jump.dest).as_str());
                    lines += 1;
                    next = cur_jump.dest;
                }
            } else {
                if cur_jump.multiple_destinations {
                    if already_visited {
                        result.push_str(format!("{}{} (multiple destinations, already visited)\n", Self::INDENT_STR.repeat(indent), cur_jump.name).as_str());
                        lines += 1;
                        return JumpTreeData { lines, result };
                    }

                    let mut paths: Vec<_> = cur_jump.destinations
                        .iter()
                        .map(|dest| {
                            let data = self.calc_jumps_tree(indent + 1, *dest, &mut parents, highlight_addrs);
                            MultipleDestinationsData { dest: *dest, lines: data.lines, result: data.result}
                        })
                        .collect();

                    paths.sort_by_key(|i| i.lines);

                    result.push_str(format!("{}{} (multiple destinations)\n", Self::INDENT_STR.repeat(indent), cur_jump.name).as_str());
                    lines += 1;
                    for (index, val) in paths.iter().enumerate() {
                        result.push_str(format!("{}destination {}: {:#06x}\n{}", Self::INDENT_STR.repeat(indent), index + 1, val.dest, val.result).as_str());
                        lines += val.lines + 1;
                    }

                    return JumpTreeData { lines, result };
                } else {
                    result.push_str(Self::INDENT_STR.repeat(indent).as_str());
                    result.push_str(cur_jump.get_name().as_str());
                    lines += 1;
                    next = cur_jump.dest;
                }
            }

            // todo: branching from jp (hl) instructions with destinations targets
            // todo: prevent infinite loops? maybe already done.

            if already_visited {
                result.push_str(" (already visited)\n");
                return JumpTreeData { lines, result };
            } else {
                result.push('\n');
            }

            last_addr = next;
            cur_jump_o = self.jumps.get_or_next(&next).unwrap();
        }

        JumpTreeData { lines, result }
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
    multiple_destinations: bool,
    destinations: HashSet<u16>,
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
            multiple_destinations: false,
            destinations: HashSet::with_capacity(0),
        }
    }

    pub fn new_not_conditional(source: u16, dest: u16, instruction_name: &'static str) -> Self {
        JumpData {
            source,
            dest,
            name: format!("{source:#06x}: {instruction_name}"),
            conditional: false,
            jump_taken: true,
            jump_skipped: false,
            multiple_destinations: false,
            destinations: HashSet::with_capacity(0),
        }
    }

    pub fn update_destinations(&mut self, new_dest: u16) {
        if self.multiple_destinations || new_dest != self.dest {
            if !self.multiple_destinations {
                self.destinations.insert(self.dest);
                // half-measure to indicate the destination is no longer good. Should use an option but don't want to refactor that.
                self.dest = 0xdbad;
                self.multiple_destinations = true;
            }
    
            if !self.destinations.contains(&new_dest) {
                self.destinations.insert(new_dest);
            }
        }
    }

    pub fn get_name(&self) -> String {
        if !self.multiple_destinations && !self.conditional {
            format!("{} -> {:#06x}", self.name, self.dest)
        } else {
            self.name.clone()
        }
    }
}
