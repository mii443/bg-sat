use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub id: i32,
    pub name: String,
}

impl Variable {
    pub fn positive(&self) -> i32 {
        self.id
    }

    pub fn negative(&self) -> i32 {
        -self.id
    }
}

pub struct CnfBuilder {
    variables: HashMap<String, Variable>,
    clauses: Vec<Vec<i32>>,
    variable_counter: i32,
}

impl CnfBuilder {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            clauses: Vec::new(),
            variable_counter: 1,
        }
    }

    pub fn clause_count(&self) -> usize {
        self.clauses.len()
    }

    pub fn get_clauses(&self) -> &Vec<Vec<i32>> {
        &self.clauses
    }

    pub fn variable(&mut self, name: String) -> Variable {
        if let Some(var) = self.variables.get(&name) {
            (*var).clone()
        } else {
            let var = Variable {
                id: self.variable_counter,
                name: name.clone(),
            };
            self.variables.insert(name, var.clone());
            self.variable_counter += 1;
            var
        }
    }

    pub fn variables(&mut self, base_name: &str, range: std::ops::Range<i32>) -> Vec<Variable> {
        range
            .map(|i| self.variable(format!("{}{}", base_name, i)))
            .collect()
    }

    pub fn clause(&mut self, literals: &[i32]) -> &mut Self {
        self.clauses.push(literals.to_vec());
        self
    }

    pub fn exactly_k_true(&mut self, k: usize, vars: &[Variable]) -> &mut Self {
        self.at_least_k_true(k, vars);
        self.at_most_k_true(k, vars);
        self
    }

    pub fn exactly_one_true(&mut self, vars: &[Variable]) -> &mut Self {
        self.exactly_k_true(1, vars)
    }

    pub fn at_most_k_true(&mut self, k: usize, vars: &[Variable]) -> &mut Self {
        if k + 1 <= vars.len() {
            for combo in combinations(vars, k + 1) {
                let clause: Vec<i32> = combo.iter().map(|v| v.negative()).collect();
                self.clause(&clause);
            }
        }

        self
    }

    pub fn at_least_k_true(&mut self, k: usize, vars: &[Variable]) -> &mut Self {
        if k == 0 {
            return self;
        }

        if k <= vars.len() {
            for i in 0..k {
                for combo in combinations(vars, i) {
                    let clause: Vec<i32> = combo.iter().map(|v| v.negative()).collect();

                    let rest: Vec<i32> = vars
                        .iter()
                        .filter(|v| !combo.contains(v))
                        .map(|v| v.positive())
                        .collect();

                    let clause: Vec<i32> = [clause, rest].concat();
                    self.clause(&clause);
                }
            }
        } else {
            self.clause(&[]);
        }
        self
    }

    pub fn to_dimacs(&self) -> String {
        let mut result = String::new();

        result.push_str(&format!(
            "p cnf {} {}\n",
            self.variable_counter - 1,
            self.clauses.len()
        ));

        let mut sorted_vars = self.variables.values().cloned().collect::<Vec<_>>();
        sorted_vars.sort_by_key(|v| v.id);

        for var in sorted_vars {
            result.push_str(&format!("c {}: {}\n", var.name, var.id));
        }

        for clause in &self.clauses {
            for &literal in clause {
                result.push_str(&format!("{} ", literal));
            }
            result.push_str("0\n");
        }

        result
    }
}

fn combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.is_empty() {
        return vec![];
    }

    let mut result = combinations(&items[1..], k - 1);
    for combo in &mut result {
        combo.insert(0, items[0].clone());
    }

    result.extend(combinations(&items[1..], k));
    result
}
