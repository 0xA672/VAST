#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_zero() {
        let input = "\tmov\trax, 0\n\tmov\trbx, 0\n\tret";
        let expected = "\txor\teax, eax\n\txor\tebx, ebx\n\tret";
        assert_eq!(xor_zero(input), expected);
    }
}
