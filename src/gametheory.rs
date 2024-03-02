use rand::distributions::{Bernoulli, Distribution};
use grid::Grid;
use std::collections::HashMap;
use std::ops::Not;

/// Outcome scores for both players based on their decisions in a game iteration.
type RewardFunc = fn(&Decision, &Decision) -> (i32, i32);

#[derive(Clone)]
pub struct Player {
    /// Stores own previous move towards players keyed by a String, values initialised to None.
    prev_move_self: HashMap<String, Option<Decision>>,
    /// Stores other players decisions towards self, same storage.
    prev_move_other: HashMap<String, Option<Decision>>,
    /// Strategy function.
    strategy: DecisionTable,
    /// Name of used player strategy.
    strategy_name: String,
}

pub struct Tournament {
    /// Players in the game.
    players: Box<[Player]>,
    /// 10x10 grid where each tuple represents (player vertical score, player horizontal score).
    scores: Grid<(i32, i32)>,
    /// Number of times to apply the [`RewardFunc`].
    max_iter: u32,
    /// How many times we've applied the [`RewardFunc`].
    current_iter: u32,
    /// What the reward function is.
    rewardsystem: RewardFunc,
}

impl Tournament {
    /// Create a new [`Tournament`].
    pub fn from(n_iter: u32, rules: RewardFunc) -> Self {
        let score_grid = Grid::from_vec(vec![(0, 0); 100], 10);
        let player_init_data: [(&str, DecisionTable); 10] = [
            ("trusting tit for tat", good_tit_for_tat),
            ("suspicious tit for tat", sus_tit_for_tat),
            ("naive", naive),
            ("evil", evil),
            ("random", random),
            ("xor logic", xor),
            ("opposite tit for tat", opposite_tit_for_tat),
            ("xnor logic", xnor),
            ("nand logic", nand),
            ("Bernoulli uncooperative", random_biased),
        ];

        let players: Vec<Player> = player_init_data
            .iter()
            .map(|(name, table)| {
                let mut initial_player_memory: HashMap<String, Option<Decision>> = HashMap::new();
                for (opponent_name, _) in player_init_data {
                    initial_player_memory.insert(opponent_name.to_owned(), None);
                }
                let memory_of_opponents = initial_player_memory.clone();
                let p = Player {
                    prev_move_self: initial_player_memory,
                    prev_move_other: memory_of_opponents,
                    strategy: *table,
                    strategy_name: name.to_string(),
                };
                p
            })
            .collect();

        Tournament {
            players: players.into_boxed_slice(),
            scores: score_grid,
            max_iter: n_iter,
            current_iter: 0,
            rewardsystem: rules,
        }
    }

    /// Runs one single simulation step, returning whether it finished.
    pub fn step(&mut self) -> bool {
        if self.current_iter == self.max_iter {
            return true;
        }
        let mut upperlim = 1;
        let mut opponents = self.players.clone();
        for j in 0..10 {
            for i in 0..upperlim {
                let player = &mut self.players[i];
                let opponent = &mut opponents[j];

                // Get decisions.
                let player_decision = (player.strategy)(
                    player
                        .prev_move_self
                        .get(&opponent.strategy_name)
                        .expect("player memory should be complete")
                        .clone(),
                    player
                        .prev_move_other
                        .get(&opponent.strategy_name)
                        .expect("player memory should be complete")
                        .clone(),
                );
                let opponent_decision = (opponent.strategy)(
                    opponent
                        .prev_move_self
                        .get(&player.strategy_name)
                        .expect("player memory should be complete")
                        .clone(),
                    opponent
                        .prev_move_other
                        .get(&player.strategy_name)
                        .expect("player memory should be complete")
                        .clone(),
                );

                // Calculate score.
                let (n, m) = (self.rewardsystem)(&opponent_decision, &player_decision);
                let (opponent_score, player_score) = self.scores[(i, j)];
                self.scores[(i, j)] = (opponent_score + n, player_score + m);

                // Update memories.
                if let None = player.prev_move_self.remove(&opponent.strategy_name) {
                    panic!("player memory should be complete")
                }
                player
                    .prev_move_self
                    .insert(opponent.strategy_name.clone(), Some(player_decision));
                if let None = player.prev_move_other.remove(&opponent.strategy_name) {
                    panic!("player memory should be complete")
                }
                player
                    .prev_move_other
                    .insert(opponent.strategy_name.clone(), Some(opponent_decision));
                // ----------------

                if let None = opponent.prev_move_self.remove(&player.strategy_name) {
                    panic!("player memory should be complete")
                }
                opponent
                    .prev_move_self
                    .insert(player.strategy_name.clone(), Some(opponent_decision));
                if let None = opponent.prev_move_other.remove(&player.strategy_name) {
                    panic!("player memory should be complete")
                }
                opponent
                    .prev_move_other
                    .insert(player.strategy_name.clone(), Some(player_decision));
            }
            upperlim += 1;
        }

        self.current_iter += 1;
        false
    }

    pub fn scores(&self) -> &Grid<(i32, i32)> {
        &self.scores
    }
}

pub fn prisoners_dillemma_rules(p1move: &Decision, p2move: &Decision) -> (i32, i32) {
    use Decision::*;
    match (p1move, p2move) {
        (Cooperate, Cooperate) => (-1, -1),
        (Cooperate, Defect) => (-3, 0),
        (Defect, Cooperate) => (0, -3),
        (Defect, Defect) => (-2, -2),
    }
}

#[derive(Clone, Copy)]
pub enum Decision {
    Cooperate,
    Defect,
}

impl Not for Decision {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Cooperate => Self::Defect,
            Self::Defect => Self::Cooperate,
        }
    }
}

pub type DecisionTable = fn(Option<Decision>, Option<Decision>) -> Decision;

pub fn good_tit_for_tat(
    _own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>,
) -> Decision {
    use Decision::*;
    match other_prev_move {
        None => Cooperate,
        Some(mv) => match mv {
            Cooperate => Cooperate,
            Defect => Defect,
        },
    }
}

pub fn sus_tit_for_tat(
    _own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>,
) -> Decision {
    use Decision::*;
    match other_prev_move {
        None => Defect,
        Some(mv) => match mv {
            Cooperate => Cooperate,
            Defect => Defect,
        },
    }
}

pub fn naive(_own_prev_move: Option<Decision>, _other_prev_move: Option<Decision>) -> Decision {
    Decision::Cooperate
}

pub fn evil(_own_prev_move: Option<Decision>, _other_prev_move: Option<Decision>) -> Decision {
    Decision::Defect
}

pub fn random(_own_prev_move: Option<Decision>, _other_prev_move: Option<Decision>) -> Decision {
    let dist = Bernoulli::new(0.5).unwrap();
    let res = dist.sample(&mut rand::thread_rng());
    match res {
        true => Decision::Cooperate,
        false => Decision::Defect,
    }
}

pub fn xor(own_prev_move: Option<Decision>, other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match (own_prev_move, other_prev_move) {
        (None, None) => Cooperate,
        (Some(own_pm), Some(other_pm)) => match (own_pm, other_pm) {
            (Cooperate, Cooperate) => Defect,
            (Cooperate, Defect) => Cooperate,
            (Defect, Cooperate) => Cooperate,
            (Defect, Defect) => Defect,
        },
        (Some(_), None) | (None, Some(_)) => unreachable!("impossible move compination"),
    }
}

pub fn opposite_tit_for_tat(
    own_prev_move: Option<Decision>,
    other_prev_move: Option<Decision>,
) -> Decision {
    !good_tit_for_tat(own_prev_move, other_prev_move)
}

pub fn xnor(own_prev_move: Option<Decision>, other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match (own_prev_move, other_prev_move) {
        (None, None) => Cooperate,
        (Some(own_pm), Some(other_pm)) => match (own_pm, other_pm) {
            (Defect, Defect) => Cooperate,
            (Cooperate, Defect) => Defect,
            (Defect, Cooperate) => Defect,
            (Cooperate, Cooperate) => Cooperate,
        },
        (Some(_), None) | (None, Some(_)) => unreachable!("impossible move compination"),
    }
}

/// No longer a strat on its own just a helper for the nand.
fn and(own_prev_move: Option<Decision>, other_prev_move: Option<Decision>) -> Decision {
    use Decision::*;
    match (own_prev_move, other_prev_move) {
        (None, None) => Cooperate,
        (Some(own_pm), Some(other_pm)) => match (own_pm, other_pm) {
            (Cooperate, Cooperate) => Cooperate,
            (Cooperate, Defect) => Defect,
            (Defect, Cooperate) => Defect,
            (Defect, Defect) => Defect,
        },
        (Some(_), None) | (None, Some(_)) => unreachable!("impossible move compination"),
    }
}

pub fn nand(own_prev_move: Option<Decision>, other_prev_move: Option<Decision>) -> Decision {
    !and(own_prev_move, other_prev_move)
}

pub fn random_biased(
    _own_prev_move: Option<Decision>,
    _other_prev_move: Option<Decision>,
) -> Decision {
    let dist = Bernoulli::new(0.3).unwrap();
    let res = dist.sample(&mut rand::thread_rng());
    match res {
        true => Decision::Cooperate,
        false => Decision::Defect,
    }
}
