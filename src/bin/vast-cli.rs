use vast::xor_zero;

fn main() {
    let input = "\tmov\trax, 0\n\tmov\trbx, 0\n\tret";
    let output = xor_zero(input);
    println!("{}", output);
}
