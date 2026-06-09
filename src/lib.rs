// lib.rs ── VAST optimization passes (migrated from VAS's opt.go)

use regex::Regex;

/// Replaces `mov reg, 0` with `xor reg, reg` (32‑bit form).
pub fn xor_zero(input: &str) -> String {
    let re = Regex::new(r"^\s*mov\s+(\w+),\s*0\s*$").unwrap();
    input
        .lines()
        .map(|line| {
            if let Some(caps) = re.captures(line) {
                let reg = &caps[1];
                format!("\txor\t{}, {}", reg32(reg), reg32(reg))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Replaces `cmp reg, 0` with `test reg, reg` (32‑bit form).
pub fn test_cmp(input: &str) -> String {
    let re = Regex::new(r"^\s*cmp\s+(\w+),\s*0\s*$").unwrap();
    input
        .lines()
        .map(|line| {
            if let Some(caps) = re.captures(line) {
                let reg = &caps[1];
                format!("\ttest\t{}, {}", reg32(reg), reg32(reg))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Fuses `mov r1, r2` followed by `add r1, r3` into `lea r1, [r2+r3]`.
/// Also handles `sub r1, imm` and `imul r1, 3/5/9`.
pub fn lea_fuse(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mov_re = Regex::new(r"^\s*mov\s+(\w+),\s*(\w+)\s*$").unwrap();
    let add_re = Regex::new(r"^\s*add\s+(\w+),\s*(\w+)\s*$").unwrap();
    let sub_imm_re = Regex::new(r"^\s*sub\s+(\w+),\s*(-?\d+)\s*$").unwrap();
    let imul_imm_re = Regex::new(r"^\s*imul\s+(\w+),\s*(\d+)\s*$").unwrap();

    let mut i = 0;
    while i < lines.len() {
        if i + 1 < lines.len() {
            let line1 = lines[i];
            let line2 = lines[i + 1];
            if let Some(caps1) = mov_re.captures(line1) {
                let dst = caps1.get(1).unwrap().as_str();
                let src1 = caps1.get(2).unwrap().as_str();

                // mov + add
                if let Some(caps2) = add_re.captures(line2) {
                    let add_dst = caps2.get(1).unwrap().as_str();
                    let src2 = caps2.get(2).unwrap().as_str();
                    if add_dst == dst {
                        result.push(format!("\tlea\t{}, [{}+{}]", dst, src1, src2));
                        i += 2;
                        continue;
                    }
                }
                // mov + sub imm
                if let Some(caps2) = sub_imm_re.captures(line2) {
                    let sub_dst = caps2.get(1).unwrap().as_str();
                    let imm = caps2.get(2).unwrap().as_str();
                    if sub_dst == dst {
                        result.push(format!("\tlea\t{}, [{}-{}]", dst, src1, imm));
                        i += 2;
                        continue;
                    }
                }
                // mov + imul imm
                if let Some(caps2) = imul_imm_re.captures(line2) {
                    let imul_dst = caps2.get(1).unwrap().as_str();
                    if imul_dst == dst {
                        let k_str = caps2.get(2).unwrap().as_str();
                        if let Ok(k) = k_str.parse::<i32>() {
                            let scale = k - 1;
                            if scale == 1 || scale == 2 || scale == 4 || scale == 8 {
                                result
                                    .push(format!("\tlea\t{}, [{}+{}*{}]", dst, src1, src1, scale));
                                i += 2;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        result.push(lines[i].to_string());
        i += 1;
    }
    result.join("\n")
}

/// Converts `push r1; pop r2` into `mov r2, r1`.
pub fn push_pop_mov(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let push_re = Regex::new(r"^\s*push\s+(\w+)\s*$").unwrap();
    let pop_re = Regex::new(r"^\s*pop\s+(\w+)\s*$").unwrap();
    let mut i = 0;
    while i < lines.len() {
        if i + 1 < lines.len() {
            if let Some(caps1) = push_re.captures(lines[i]) {
                let src = caps1.get(1).unwrap().as_str();
                if let Some(caps2) = pop_re.captures(lines[i + 1]) {
                    let dst = caps2.get(1).unwrap().as_str();
                    result.push(format!("\tmov\t{}, {}", dst, src));
                    i += 2;
                    continue;
                }
            }
        }
        result.push(lines[i].to_string());
        i += 1;
    }
    result.join("\n")
}

/// Removes no‑op instructions: `mov r1, r1`, `add r1, 0`, `sub r1, 0`, `imul r1, 1`.
pub fn noop_elim(input: &str) -> String {
    let mov_re = Regex::new(r"^\s*mov\s+(\w+),\s*(\w+)\s*$").unwrap();
    let add_zero_re = Regex::new(r"^\s*add\s+(\w+),\s*0\s*$").unwrap();
    let sub_zero_re = Regex::new(r"^\s*sub\s+(\w+),\s*0\s*$").unwrap();
    let imul_one_re = Regex::new(r"^\s*imul\s+(\w+),\s*1\s*$").unwrap();

    input
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if let Some(caps) = mov_re.captures(trimmed) {
                // Keep only if the two operands are different
                return caps.get(1).unwrap().as_str() != caps.get(2).unwrap().as_str();
            }
            !add_zero_re.is_match(trimmed)
                && !sub_zero_re.is_match(trimmed)
                && !imul_one_re.is_match(trimmed)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Removes back‑to‑back canceling pairs:
///   `not r1; not r1` → deleted
///   `neg r1; neg r1` → deleted
///   `inc r1; dec r1` → deleted
///   `dec r1; inc r1` → deleted
pub fn cancel_pair_elim(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let not_re = Regex::new(r"^\s*not\s+(\w+)\s*$").unwrap();
    let neg_re = Regex::new(r"^\s*neg\s+(\w+)\s*$").unwrap();
    let inc_re = Regex::new(r"^\s*inc\s+(\w+)\s*$").unwrap();
    let dec_re = Regex::new(r"^\s*dec\s+(\w+)\s*$").unwrap();

    let mut i = 0;
    while i < lines.len() {
        if i + 1 < lines.len() {
            let line = lines[i];
            let next = lines[i + 1];
            // not r1; not r1
            if let (Some(c1), Some(c2)) = (not_re.captures(line), not_re.captures(next)) {
                if c1.get(1).unwrap().as_str() == c2.get(1).unwrap().as_str() {
                    i += 2;
                    continue;
                }
            }
            // neg r1; neg r1
            if let (Some(c1), Some(c2)) = (neg_re.captures(line), neg_re.captures(next)) {
                if c1.get(1).unwrap().as_str() == c2.get(1).unwrap().as_str() {
                    i += 2;
                    continue;
                }
            }
            // inc r1; dec r1
            if let (Some(c1), Some(c2)) = (inc_re.captures(line), dec_re.captures(next)) {
                if c1.get(1).unwrap().as_str() == c2.get(1).unwrap().as_str() {
                    i += 2;
                    continue;
                }
            }
            // dec r1; inc r1
            if let (Some(c1), Some(c2)) = (dec_re.captures(line), inc_re.captures(next)) {
                if c1.get(1).unwrap().as_str() == c2.get(1).unwrap().as_str() {
                    i += 2;
                    continue;
                }
            }
        }
        result.push(lines[i].to_string());
        i += 1;
    }
    result.join("\n")
}

/// Deletes `push reg; modify reg; pop reg` triples when the intermediate
/// result is unused (i.e., the pop restores the old value).
pub fn push_mod_pop_elim(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let push_re = Regex::new(r"^\s*push\s+(\w+)\s*$").unwrap();
    let pop_re = Regex::new(r"^\s*pop\s+(\w+)\s*$").unwrap();
    let mod_re = Regex::new(r"^\s*(add|sub|imul|mov|lea)\s+(\w+),.*$").unwrap();

    let mut i = 0;
    while i < lines.len() {
        if i + 2 < lines.len() {
            if let Some(caps1) = push_re.captures(lines[i]) {
                let reg1 = caps1.get(1).unwrap().as_str();
                if let Some(caps3) = pop_re.captures(lines[i + 2]) {
                    let reg3 = caps3.get(1).unwrap().as_str();
                    if reg1 == reg3 {
                        if let Some(caps2) = mod_re.captures(lines[i + 1]) {
                            let mod_reg = caps2.get(2).unwrap().as_str();
                            if mod_reg == reg1 {
                                // push r1; mod r1; pop r1 → delete
                                i += 3;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        result.push(lines[i].to_string());
        i += 1;
    }
    result.join("\n")
}

// helper: 64‑bit → 32‑bit register name
fn reg32(reg64: &str) -> &str {
    match reg64 {
        "rax" => "eax",
        "rbx" => "ebx",
        "rcx" => "ecx",
        "rdx" => "edx",
        "rsi" => "esi",
        "rdi" => "edi",
        "rbp" => "ebp",
        "rsp" => "esp",
        "r8" => "r8d",
        "r9" => "r9d",
        "r10" => "r10d",
        "r11" => "r11d",
        "r12" => "r12d",
        "r13" => "r13d",
        "r14" => "r14d",
        "r15" => "r15d",
        _ => reg64,
    }
}
