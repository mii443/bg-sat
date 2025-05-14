use crate::dimacs::{self, CnfBuilder, Variable};

static CARDS: i32 = 9;

pub struct NumComp {}

impl NumComp {
    pub fn new() -> Self {
        Self {}
    }

    pub fn to_dimacs(&self) -> String {
        todo!()
    }

    pub fn generate_base_dimacs(&self) -> CnfBuilder {
        let mut cnf = dimacs::CnfBuilder::new();

        let self_cards_per_round: Vec<Vec<Variable>> = (0..CARDS)
            .into_iter()
            .map(|r| cnf.variables(&format!("self_card_{}_", r), 0..CARDS))
            .collect();

        let opponent_cards_per_round: Vec<Vec<Variable>> = (0..CARDS)
            .into_iter()
            .map(|r| cnf.variables(&format!("opponent_card_{}_", r), 0..CARDS))
            .collect();

        let round_info: Vec<Vec<Variable>> = (0..CARDS)
            .into_iter()
            .map(|r| cnf.variables(&format!("round_info_{}_", r), 0..CARDS))
            .collect();

        let opponent_round_info = cnf.variables("opponent_round_info", 0..CARDS);

        let round_win = cnf.variables("round_win", 0..CARDS);
        let round_lose = cnf.variables("round_lose", 0..CARDS);
        let round_draw = cnf.variables("round_draw", 0..CARDS);

        for round in 0..CARDS as usize {
            for card in 0..CARDS as usize {
                if (card % 2) == 0 {
                    Self::gen_dimacs_a_and_b_implies_c_and_not_d(
                        &mut cnf,
                        &round_info[round][card],
                        &round_draw[round],
                        &opponent_round_info[round],
                        &opponent_cards_per_round[round][card],
                    );
                } else {
                    Self::gen_dimacs_a_and_b_implies_not_c_and_not_d(
                        &mut cnf,
                        &round_info[round][card],
                        &round_draw[round],
                        &opponent_round_info[round],
                        &opponent_cards_per_round[round][card],
                    );
                }

                cnf.right_negative_implies(
                    &round_info[round][card],
                    &self_cards_per_round[round][card],
                );

                if round != 0 {
                    cnf.negative_implies(
                        &self_cards_per_round[round - 1][card],
                        &self_cards_per_round[round][card],
                    );
                    cnf.negative_implies(
                        &opponent_cards_per_round[round - 1][card],
                        &opponent_cards_per_round[round][card],
                    );

                    let opponent_cards =
                        Self::get_cards_under_k(&opponent_cards_per_round[round], card + 1, true);
                    if !opponent_cards.is_empty() {
                        cnf.at_most_k_true_with_two_implies(
                            opponent_cards.len() - 1,
                            &opponent_cards,
                            &[
                                round_info[round][card].positive(),
                                round_win[round].positive(),
                                opponent_round_info[round].positive(),
                            ],
                        );
                    }

                    let opponent_cards =
                        Self::get_cards_under_k(&opponent_cards_per_round[round], card + 1, false);
                    if !opponent_cards.is_empty() {
                        cnf.at_most_k_true_with_two_implies(
                            opponent_cards.len() - 1,
                            &opponent_cards,
                            &[
                                round_info[round][card].positive(),
                                round_win[round].positive(),
                                opponent_round_info[round].negative(),
                            ],
                        );
                    }

                    let opponent_cards =
                        Self::get_cards_over_k(&opponent_cards_per_round[round], card + 1, true);
                    if !opponent_cards.is_empty() {
                        cnf.at_most_k_true_with_two_implies(
                            opponent_cards.len() - 1,
                            &opponent_cards,
                            &[
                                round_info[round][card].negative(),
                                round_lose[round].positive(),
                                opponent_round_info[round].positive(),
                            ],
                        );
                    }

                    let opponent_cards =
                        Self::get_cards_over_k(&opponent_cards_per_round[round], card + 1, false);
                    if !opponent_cards.is_empty() {
                        cnf.at_most_k_true_with_two_implies(
                            opponent_cards.len() - 1,
                            &opponent_cards,
                            &[
                                round_info[round][card].negative(),
                                round_lose[round].positive(),
                                opponent_round_info[round].negative(),
                            ],
                        );
                    }

                    if card == 0 {
                        // (round_info[round][card] and round_win[round] and opponent_round_info[round]) -> opponent_cards_per_round[round][8]
                        cnf.clause(&[
                            round_info[round][card].negative(),
                            round_win[round].negative(),
                            opponent_round_info[round].negative(),
                            opponent_cards_per_round[round][8].positive(),
                        ]);
                    }

                    if card == 8 {
                        // (round_info[round][card] and round_lose[round] and opponent_round_info[round]) -> opponent_cards_per_round[round][0]
                        cnf.clause(&[
                            round_info[round][card].negative(),
                            round_lose[round].negative(),
                            opponent_round_info[round].negative(),
                            opponent_cards_per_round[round][0].positive(),
                        ]);
                    }
                }
            }

            cnf.exactly_k_true(
                (CARDS - 1 - round as i32) as usize,
                &opponent_cards_per_round[round],
            );
            cnf.exactly_k_true(
                (CARDS - 1 - round as i32) as usize,
                &self_cards_per_round[round],
            );
        }

        cnf
    }

    fn get_cards_under_k(cards: &[Variable], k: usize, is_odd: bool) -> Vec<Variable> {
        cards
            .iter()
            .enumerate()
            .filter(|(i, _)| (i + 1) < k && ((i % 2 != 0) ^ is_odd))
            .map(|(_, v)| v.clone())
            .collect()
    }

    fn get_cards_over_k(cards: &[Variable], k: usize, is_odd: bool) -> Vec<Variable> {
        cards
            .iter()
            .enumerate()
            .filter(|(i, _)| (i + 1) > k && ((i % 2 != 0) ^ is_odd))
            .map(|(_, v)| v.clone())
            .collect()
    }

    fn gen_dimacs_a_and_b_implies_c_and_not_d(
        cnf: &mut CnfBuilder,
        a: &Variable,
        b: &Variable,
        c: &Variable,
        d: &Variable,
    ) {
        let tmp = cnf.get_tmp_variable();
        cnf.clause(&[a.negative(), b.negative(), tmp.positive()]);
        cnf.clause(&[tmp.negative(), c.positive()]);
        cnf.clause(&[tmp.negative(), d.negative()]);
        cnf.clause(&[c.negative(), d.positive(), tmp.positive()]);
    }

    fn gen_dimacs_a_and_b_implies_not_c_and_not_d(
        cnf: &mut CnfBuilder,
        a: &Variable,
        b: &Variable,
        c: &Variable,
        d: &Variable,
    ) {
        let tmp = cnf.get_tmp_variable();
        cnf.clause(&[a.negative(), b.negative(), tmp.positive()]);
        cnf.clause(&[tmp.negative(), c.negative()]);
        cnf.clause(&[tmp.negative(), d.negative()]);
        cnf.clause(&[c.positive(), d.positive(), tmp.positive()]);
    }
}
