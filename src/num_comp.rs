use crate::dimacs::{self, CnfBuilder, Variable};

pub struct NumComp {}

impl NumComp {
    pub fn new() -> Self {
        Self {}
    }

    pub fn to_dimacs(&self) -> String {
        todo!()
    }

    pub fn generate_base_dimacs(&self) -> String {
        let mut cnf = dimacs::CnfBuilder::new();

        // カード情報
        let self_cards = cnf.variables("self_card", 0..9);
        let opponent_cards = cnf.variables("opponent_card", 0..9);

        // ラウンドの情報
        let round_info = cnf.variables("round_info", 0..9 * 9);

        // 相手のラウンドの情報
        let opponent_round_info = cnf.variables("opponent_round_info", 0..9);

        // ラウンドの勝敗
        let round_win = cnf.variables("round_win", 0..9);
        let round_lose = cnf.variables("round_lose", 0..9);
        let round_draw = cnf.variables("round_draw", 0..9);

        // 各ラウンド
        for round in 0..9 {
            /* nを出し、引き分けの場合、相手は対応したカードを出し、相手はカードnを持っていない */
            for card in 0..9 {
                if (card % 2) == 0 {
                    Self::gen_dimacs_a_and_b_implies_c_and_not_d(
                        &mut cnf,
                        &round_info[round * 9 + card],
                        &round_draw[round],
                        &opponent_round_info[round],
                        &opponent_cards[card],
                    );
                } else {
                    Self::gen_dimacs_a_and_b_implies_not_c_and_not_d(
                        &mut cnf,
                        &round_info[round * 9 + card],
                        &round_draw[round],
                        &opponent_round_info[round],
                        &opponent_cards[card],
                    );
                }
            }
        }

        cnf.clause(&[round_draw[0].positive()]);
        cnf.clause(&[round_info[0].positive()]);

        cnf.to_dimacs()
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
