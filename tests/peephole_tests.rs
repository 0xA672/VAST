// tests/peephole_tests.rs

use vast::{
    cancel_pair_elim, lea_fuse, noop_elim, push_mod_pop_elim, push_pop_mov, test_cmp, xor_zero,
};

#[test]
fn test_xor_zero() {
    let input = "\tmov\trax, 0\n\tmov\trbx, 0\n\tret";
    let expected = "\txor\teax, eax\n\txor\tebx, ebx\n\tret";
    assert_eq!(xor_zero(input), expected);
}

#[test]
fn test_cmp_to_test() {
    let input = "\tcmp\trax, 0\n\tcmp\trbx, 0\n\tret";
    let expected = "\ttest\teax, eax\n\ttest\tebx, ebx\n\tret";
    assert_eq!(test_cmp(input), expected);
}

#[test]
fn test_lea_fuse_add() {
    let input = "\tmov\trax, rbx\n\tadd\trax, rcx";
    let expected = "\tlea\trax, [rbx+rcx]";
    assert_eq!(lea_fuse(input), expected);
}

#[test]
fn test_lea_fuse_sub_imm() {
    let input = "\tmov\trax, rbx\n\tsub\trax, 5";
    let expected = "\tlea\trax, [rbx-5]";
    assert_eq!(lea_fuse(input), expected);
}

#[test]
fn test_lea_fuse_imul_3() {
    let input = "\tmov\trax, rbx\n\timul\trax, 3";
    let expected = "\tlea\trax, [rbx+rbx*2]";
    assert_eq!(lea_fuse(input), expected);
}

#[test]
fn test_push_pop_mov() {
    let input = "\tpush\trax\n\tpop\trbx";
    let expected = "\tmov\trbx, rax";
    assert_eq!(push_pop_mov(input), expected);
}

#[test]
fn test_noop_elim() {
    let input = "\tmov\trax, rax\n\tadd\trax, 0\n\tsub\trbx, 0\n\timul\trcx, 1\n\tmov\trdx, rsi";
    let expected = "\tmov\trdx, rsi";
    assert_eq!(noop_elim(input), expected);
}

#[test]
fn test_cancel_pair_not() {
    let input = "\tnot\trax\n\tnot\trax";
    let expected = "";
    assert_eq!(cancel_pair_elim(input), expected);
}

#[test]
fn test_cancel_pair_inc_dec() {
    let input = "\tinc\trbx\n\tdec\trbx";
    let expected = "";
    assert_eq!(cancel_pair_elim(input), expected);
}

#[test]
fn test_push_mod_pop_elim() {
    let input = "\tpush\trbx\n\tadd\trbx, r8\n\tpop\trbx";
    let expected = "";
    assert_eq!(push_mod_pop_elim(input), expected);
}
