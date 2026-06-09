/// Replaces `mov reg, 0` with `xor reg, reg` (32-bit form).
pub fn xor_zero(input: &str) -> String {
    let re = regex::Regex::new(r"^\s*mov\s+(\w+),\s*0\s*$").unwrap();
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
