pub mod dimacs;
pub mod num_comp;
fn main() {
    let num_comp = num_comp::NumComp::new();
    let cnf = num_comp.generate_base_dimacs();
    println!("{}", cnf.to_dimacs());
}
