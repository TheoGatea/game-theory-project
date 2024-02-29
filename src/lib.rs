use rand::distributions::{Bernoulli, Distribution};

pub enum Decision {
    Cooperate,
    Defect
}

pub type DecisionTable = fn(Option<Decision>, Option<Decision>) -> Decision;

pub struct Player {
    prev_move_self: Option<Decision>,
    prev_move_other: Option<Decision>,
    strategy: DecisionTable
}

pub fn tit_for_tat(_own_prev_move: Option<Decision>,
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