use rand::distributions::{Bernoulli, Distribution};
use grid::Grid;
use std::collections::HashMap;
use std::ops::Not;

/// Outcome scores for both players based on their decisions in a game iteration.
type RewardFunc = fn(&Decision, &Decision) -> (i32, i32);

#[derive(Clone)]
pub struct Player {
    /// Stores own previous move towards players keyed by a String, values initialised to None.
    prev_move_self: HashMap<&'static str, Option<Decision>>,
    /// Stores other players decisions towards self, same storage.
    prev_move_other: HashMap<&'static str, Option<Decision>>,
    /// Strategy function.
    strategy: DecisionTable,
    /// Name of used player strategy.
    strategy_name: &'static str,
}

impl Player {
    pub fn strategy_name(&self) -> &'static str {
        self.strategy_name
    }
}

pub struct Tournament {
    /// Players in the game.
    players: Box<[Player]>,
    /// Opponents to the players (clone of players but with separate memory)
    opponents: Box<[Player]>,
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
        static PLAYER_INIT_DATA: [(&str, DecisionTable); 10] = [
            ("trusting\nt4t", good_tit_for_tat),
            ("suspicious\nt4t", sus_tit_for_tat),
            ("naive", naive),
            ("evil", evil),
            ("random", random),
            ("xor", xor),
            ("opposite\nt4t", opposite_tit_for_tat),
            ("xnor", xnor),
            ("nand", nand),
            ("Bernoulli", random_biased),
        ];

        let score_grid = Grid::new(10, 10);
        let players: Vec<Player> = PLAYER_INIT_DATA
            .iter()
            .map(|(name, table)| {
                let mut initial_player_memory = HashMap::new();
                for (opponent_name, _) in PLAYER_INIT_DATA {
                    initial_player_memory.insert(opponent_name, None);
                }
                let memory_of_opponents = initial_player_memory.clone();
                Player {
                    prev_move_self: initial_player_memory,
                    prev_move_other: memory_of_opponents,
                    strategy: *table,
                    strategy_name: name,
                }
            })
            .collect();

        Tournament {
            players: players.clone().into_boxed_slice(),
            opponents: players.into_boxed_slice(),
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
        for j in 0..10 {
            for i in 0..upperlim {
                let player = &mut self.players[i];
                let opponent = &mut self.opponents[j];

                // Get decisions.
                let player_decision = (player.strategy)(
                    *player
                        .prev_move_self
                        .get(&opponent.strategy_name)
                        .expect("player memory should be complete"),
                    *player
                        .prev_move_other
                        .get(&opponent.strategy_name)
                        .expect("player memory should be complete")
                );
                let opponent_decision = (opponent.strategy)(
                    *opponent
                        .prev_move_self
                        .get(&player.strategy_name)
                        .expect("player memory should be complete"),
                    *opponent
                        .prev_move_other
                        .get(&player.strategy_name)
                        .expect("player memory should be complete")
                );

                // Calculate score.
                let (n, m) = (self.rewardsystem)(&opponent_decision, &player_decision);
                let (opponent_score, player_score) = self.scores[(i, j)];
                self.scores[(i, j)] = (opponent_score + n, player_score + m);

                // Update memories.
                if player.prev_move_self.remove(&opponent.strategy_name).is_none() {
                    panic!("player memory should be complete")
                }
                player
                    .prev_move_self
                    .insert(opponent.strategy_name, Some(player_decision));
                if player.prev_move_other.remove(&opponent.strategy_name).is_none() {
                    panic!("player memory should be complete")
                }
                player
                    .prev_move_other
                    .insert(opponent.strategy_name, Some(opponent_decision));
                // ----------------

                if opponent.prev_move_self.remove(&player.strategy_name).is_none() {
                    panic!("player memory should be complete")
                }
                opponent
                    .prev_move_self
                    .insert(player.strategy_name, Some(opponent_decision));
                if opponent.prev_move_other.remove(&player.strategy_name).is_none() {
                    panic!("player memory should be complete")
                }
                opponent
                    .prev_move_other
                    .insert(player.strategy_name, Some(player_decision));
            }
            upperlim += 1;
        }

        self.current_iter += 1;
        false
    }

    pub fn opponents(&self) -> &[Player] {
        &self.opponents
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn scores(&self) -> &Grid<(i32, i32)> {
        &self.scores
    }

    pub fn write_scores_to_file(&self) -> std::io::Result<()> {
        let mut upperlim = 1;
        let mut scores_map: HashMap<String, Vec<i32>> = HashMap::new();
        for j in 0..10 {
            for i in 0..upperlim {
                let player_name = self.players[i].strategy_name().replace("\n", " ");
                let opponent_name = self.opponents[j].strategy_name().replace("\n", " ");
                let (_, player_score) = self.scores()[(i, j)];
                let (opponent_score, _) = self.scores()[(i, j)];
                match scores_map.get_mut(&player_name) {
                    None => {let _ = scores_map.insert(player_name.clone(), vec![player_score]);},
                    Some(score_record) => score_record.push(player_score),
                };
                match scores_map.get_mut(&opponent_name) {
                    None => {let _ = scores_map.insert(opponent_name.clone(), vec![opponent_score]);},
                    Some(score_record) => score_record.push(opponent_score),
                };
            }
            upperlim += 1;
        }
        let mut acc = String::new();
        for (participant_name, score_vec) in scores_map.iter() {
            let average = score_vec.iter().sum::<i32>() as f32 / score_vec.len() as f32;
            let stdev = std_deviation(&average, &score_vec);
            acc.push_str(participant_name);
            acc.push(':');
            acc.push_str(&average.to_string());
            acc.push(':');
            acc.push_str(&stdev.to_string());
            acc.push('\n');
        }
        std::fs::write("tournament_results.txt", acc)
    }
}

fn std_deviation(mean: &f32, data: &[i32]) -> f32 {
    match (mean, data.len()) {
        (average, count) if count > 0 => {
            let variance = data.iter().map(|value| {
                let diff = average - (*value as f32);

                diff * diff
            }).sum::<f32>() / count as f32;

            variance.sqrt()
        },
        _ => 0.0
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
