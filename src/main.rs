pub mod dimacs;
pub mod num_comp;
fn main() {
    let num_comp = num_comp::NumComp::new();
    let dimacs = num_comp.generate_base_dimacs();
    println!("{}", dimacs);
}
