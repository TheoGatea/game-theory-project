use rand::distributions::{Bernoulli, Distribution};
use std::ops::Not;

pub enum Decision {
    Cooperate,
    Defect
}

impl Not for Decision {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Cooperate => Self::Defect,
            Self::Defect => Self::Cooperate            
        }
    }
}

pub type DecisionTable = fn(Option<Decision>, Option<Decision>) -> Decision;

pub struct Player {
    prev_move_self: Option<Decision>,
    prev_move_other: Option<Decision>,
    strategy: DecisionTable,
    strategy_name: String
}

pub fn good_tit_for_tat(_own_prev_move: Option<Decision>,
        other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match other_prev_move {
        None => Cooperate,
        Some(mv) => match mv {
            Cooperate => Cooperate,
            Defect => Defect            
        }
    }
}

pub fn sus_tit_for_tat(_own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match other_prev_move {
        None => Defect,
        Some(mv) => match mv {
            Cooperate => Cooperate,
            Defect => Defect            
        }
    }
}

pub fn naive(_own_prev_move: Option<Decision>,
    _other_prev_move: Option<Decision>) -> Decision {
    Decision::Cooperate
}

pub fn evil(_own_prev_move: Option<Decision>,
    _other_prev_move: Option<Decision>) -> Decision {
    Decision::Defect
}

pub fn random(_own_prev_move: Option<Decision>,
    _other_prev_move: Option<Decision>) -> Decision {
        let dist = Bernoulli::new(0.5).unwrap();
        let res = dist.sample(&mut rand::thread_rng());
        match res {
            true => Decision::Cooperate,
            false => Decision::Defect          
        }
    }

pub fn xor(own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match (own_prev_move, other_prev_move) {
        (None, None) => Cooperate,
        (Some(own_pm), Some(other_pm)) => match (own_pm, other_pm) {
            (Cooperate, Cooperate) => Defect,
            (Cooperate, Defect) => Cooperate,
            (Defect, Cooperate) => Cooperate,
            (Defect, Defect) => Defect
        }
        (Some(_), None) | (None, Some(_)) => panic!("impossible move compination")
    }
}

pub fn opposite_tit_for_tat(own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>) -> Decision {
    !good_tit_for_tat(own_prev_move, other_prev_move)
}

pub fn and(own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match (own_prev_move, other_prev_move) {
        (None, None) => Cooperate,
        (Some(own_pm), Some(other_pm)) => match (own_pm, other_pm) {
            (Cooperate, Cooperate) => Cooperate,
            (Cooperate, Defect) => Defect,
            (Defect, Cooperate) => Defect,
            (Defect, Defect) => Defect
        }
        (Some(_), None) | (None, Some(_)) => panic!("impossible move compination")
    }
}

pub fn nand(own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>) -> Decision {
    !and(own_prev_move, other_prev_move)
}

pub fn random_biased(_own_prev_move: Option<Decision>,
    _other_prev_move: Option<Decision>) -> Decision {
    let dist = Bernoulli::new(0.5).unwrap();
    let res = dist.sample(&mut rand::thread_rng());
    match res {
        true => Decision::Cooperate,
        false => Decision::Defect          
    }
}