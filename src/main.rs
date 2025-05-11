pub mod dimacs;

fn main() {
    let mut cnf = dimacs::CnfBuilder::new();

    let vars = cnf.variables("x", 0..40);

    cnf.at_least_k_true(5, &vars);
    cnf.at_most_k_true(5, &vars);

    println!("{}", cnf.to_dimacs());
}
