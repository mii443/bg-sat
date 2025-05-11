use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub id: i32,
    pub name: String,
}

impl Variable {
    #[inline]
    pub fn positive(&self) -> i32 {
        self.id
    }

    #[inline]
    pub fn negative(&self) -> i32 {
        -self.id
    }
}

pub struct CnfBuilder {
    variables: HashMap<String, Variable>,
    clauses: Mutex<Vec<Vec<i32>>>,
    variable_counter: i32,
}

impl CnfBuilder {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            clauses: Mutex::new(Vec::new()),
            variable_counter: 1,
        }
    }

    pub fn clause_count(&self) -> usize {
        self.clauses.lock().unwrap().len()
    }

    pub fn get_clauses(&self) -> Vec<Vec<i32>> {
        self.clauses.lock().unwrap().clone()
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

    pub fn get_tmp_variable(&mut self) -> Variable {
        let name = format!("tmp{}", self.variable_counter);
        let var = self.variable(name);
        var
    }

    pub fn variables(&mut self, base_name: &str, range: std::ops::Range<i32>) -> Vec<Variable> {
        range
            .map(|i| self.variable(format!("{}{}", base_name, i)))
            .collect()
    }

    pub fn clause(&mut self, literals: &[i32]) -> &mut Self {
        self.clauses.lock().unwrap().push(literals.to_vec());
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
            let combinations = generate_combinations_optimized(vars, k + 1);

            let new_clauses: Vec<Vec<i32>> = combinations
                .into_par_iter()
                .map(|combo| combo.iter().map(|v| v.negative()).collect())
                .collect();

            self.clauses.lock().unwrap().extend(new_clauses);
        }

        self
    }

    pub fn at_least_k_true(&mut self, k: usize, vars: &[Variable]) -> &mut Self {
        if k == 0 {
            return self;
        }

        if k <= vars.len() {
            let mut all_new_clauses = Vec::new();

            for i in 0..k {
                let all_combinations = generate_combinations_optimized(vars, i);

                let new_clauses: Vec<Vec<i32>> = all_combinations
                    .into_par_iter()
                    .map(|combo| {
                        let negated_combo: Vec<i32> = combo.iter().map(|v| v.negative()).collect();

                        let rest: Vec<i32> = vars
                            .iter()
                            .filter(|v| !combo.contains(v))
                            .map(|v| v.positive())
                            .collect();

                        [negated_combo, rest].concat()
                    })
                    .collect();

                all_new_clauses.extend(new_clauses);
            }

            self.clauses.lock().unwrap().extend(all_new_clauses);
        } else {
            self.clause(&[]);
        }
        self
    }

    pub fn to_dimacs(&self) -> String {
        let mut result = String::new();

        let clauses = self.clauses.lock().unwrap();
        result.push_str(&format!(
            "p cnf {} {}\n",
            self.variable_counter - 1,
            clauses.len()
        ));

        let mut sorted_vars = self.variables.values().cloned().collect::<Vec<_>>();
        sorted_vars.sort_by_key(|v| v.id);

        for var in sorted_vars {
            result.push_str(&format!("c {}: {}\n", var.name, var.id));
        }

        for clause in &*clauses {
            for &literal in clause {
                result.push_str(&format!("{} ", literal));
            }
            result.push_str("0\n");
        }

        result
    }

    pub fn add_clauses_batch(&mut self, clauses: Vec<Vec<i32>>) {
        self.clauses.lock().unwrap().extend(clauses);
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            variables: HashMap::new(),
            clauses: Mutex::new(Vec::with_capacity(capacity)),
            variable_counter: 1,
        }
    }
}

fn generate_combinations_optimized<T: Clone + Send + Sync>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.is_empty() {
        return vec![];
    }
    if k == 1 {
        return items.iter().map(|item| vec![item.clone()]).collect();
    }

    if items.len() < 10 {
        return combinations_sequential(items, k);
    }

    let first_element = items[0].clone();
    let remaining = &items[1..];

    let (mut with_first, without_first) = rayon::join(
        || {
            let mut combos = generate_combinations_optimized(remaining, k - 1);
            for combo in &mut combos {
                combo.insert(0, first_element.clone());
            }
            combos
        },
        || generate_combinations_optimized(remaining, k),
    );

    with_first.extend(without_first);
    with_first
}

fn combinations_sequential<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.is_empty() {
        return vec![];
    }

    let first_element = items[0].clone();
    let remaining = &items[1..];

    let mut with_first = combinations_sequential(remaining, k - 1);
    for combo in &mut with_first {
        combo.insert(0, first_element.clone());
    }

    let without_first = combinations_sequential(remaining, k);

    [with_first, without_first].concat()
}
