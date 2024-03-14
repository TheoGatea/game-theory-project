use grid::Grid;
use rand::distributions::{Bernoulli, Distribution};
use rand::Rng;
use std::collections::HashMap;
use std::ops::Not;

/// Outcome scores for both players based on their decisions in a game iteration.
type RewardFunc = fn(&Decision, &Decision) -> (i32, i32);

/// boolean array of length 5 used to compose [`DecisionTable`]'s in a softcoded way
type Genome = Box<[bool]>;

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

const GENOME_LENGTH: i32 = 5;
const POPULATION_SIZE: usize = 20;
const GENERATION_SIZE: usize = 10;

fn number_to_genome(n: u8) -> Genome {
    let mut genome = [false; GENOME_LENGTH as usize];
    let mut mask = 1;
    for i in (0..GENOME_LENGTH).rev() {
        let res = n & mask;
        if res != 0 {
            genome[i as usize] = true;
        }
        mask = mask << 1;
    }
    Box::new(genome)
}

fn genome_to_number(g: &Genome) -> u8 {
    let mut acc: u8 = 0;
    let mut exp = 0;
    for i in (0..GENOME_LENGTH).rev() {
        let n = 2_i32.pow(exp);
        if g[i as usize] {
            acc += n as u8;
        }
        exp += 1;
    }
    acc
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
    /// What the reward function is.
    rewardsystem: RewardFunc,
}

impl Tournament {
    /// Create a new [`Tournament`].
    pub fn from(n_iter: u32, rules: RewardFunc, opponent_starting_pop: Box<[u8]>) -> Self {
        static PLAYER_INIT_DATA: [(&str, fn(Option<Decision>, Option<Decision>) -> Decision); 10] = [
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
        let opponent_names: Vec<String> = (0..POPULATION_SIZE)
            .into_iter()
            .map(|n| (opponent_starting_pop[n] as i32).to_string())
            .collect();

        let players: Vec<Player> = PLAYER_INIT_DATA
            .iter()
            .map(|(name, table)| {
                let mut initial_player_memory = HashMap::new();
                for opponent_name in opponent_names.clone() {
                    initial_player_memory.insert(opponent_name.clone(), None);
                }
                let memory_of_opponents = initial_player_memory.clone();
                Player {
                    prev_move_self: initial_player_memory,
                    prev_move_other: memory_of_opponents,
                    strategy: Box::new(table),
                    strategy_name: name.to_string(),
                }
            })
            .collect();

        let opponents_selection = opponent_starting_pop
            .iter()
            .map(|&c| {
                let mut initial_opponent_memory = HashMap::new();
                for (name, _) in PLAYER_INIT_DATA {
                    initial_opponent_memory.insert(name.to_string(), None);
                }
                let memory_of_players = initial_opponent_memory.clone();
                let gene: Vec<Decision> = number_to_genome(c)
                    .iter()
                    .map(|&b| {
                        if b {
                            Decision::Cooperate
                        } else {
                            Decision::Defect
                        }
                    })
                    .collect();

                let strat: DecisionTable = Box::new(move |own_pm, other_pm| {
                    use Decision::*;
                    match (own_pm, other_pm) {
                        (None, None) => gene[0],
                        (Some(ownpm), Some(otherpm)) => match (ownpm, otherpm) {
                            (Cooperate, Cooperate) => gene[1],
                            (Cooperate, Defect) => gene[2],
                            (Defect, Cooperate) => gene[3],
                            (Defect, Defect) => gene[4],
                        },
                        (Some(_), None) | (None, Some(_)) => {
                            unreachable!("impossible move combination")
                        }
                    }
                });

                Player {
                    prev_move_self: initial_opponent_memory,
                    prev_move_other: memory_of_players,
                    strategy: strat,
                    strategy_name: (c as i32).to_string(),
                }
            })
            .collect();

        Tournament {
            players: players.into_boxed_slice(),
            opponents: opponents_selection,
            scores: Grid::new(POPULATION_SIZE, 10),
            max_iter: n_iter,
            rewardsystem: rules,
        }
    }

    fn execute_round_and_update_scores(&mut self, i: usize, j: usize) {
        let player = &mut self.players[j];
        let opponent = &mut self.opponents[i];

        // Get decisions.
        let player_decision = (player.strategy)(
            *player
                .prev_move_self
                .get(&opponent.strategy_name)
                .expect("player memory should be complete"),
            *player
                .prev_move_other
                .get(&opponent.strategy_name)
                .expect("player memory should be complete"),
        );
        let opponent_decision = (opponent.strategy)(
            *opponent
                .prev_move_self
                .get(&player.strategy_name)
                .expect("player memory should be complete"),
            *opponent
                .prev_move_other
                .get(&player.strategy_name)
                .expect("player memory should be complete"),
        );

        // Calculate score.
        let (n, m) = (self.rewardsystem)(&opponent_decision, &player_decision);
        let (opponent_score, player_score) = self.scores[(i, j)];
        self.scores[(i, j)] = (opponent_score + n, player_score + m);

        // Update memories.
        if player.prev_move_self.remove(&opponent.strategy_name).is_none() {
            panic!("player memory should be complete")
        }
        player.prev_move_self.insert(opponent.strategy_name.clone(), Some(player_decision));
        if player.prev_move_other.remove(&opponent.strategy_name).is_none() {
            panic!("player memory should be complete")
        }
        player.prev_move_other.insert(opponent.strategy_name.clone(), Some(opponent_decision));
        // ----------------

        if opponent.prev_move_self.remove(&player.strategy_name).is_none() {
            panic!("player memory should be complete")
        }
        opponent.prev_move_self.insert(player.strategy_name.clone(), Some(opponent_decision));
        if opponent.prev_move_other.remove(&player.strategy_name).is_none() {
            panic!("player memory should be complete")
        }
        opponent.prev_move_other.insert(player.strategy_name.clone(), Some(player_decision));
    }

    /// Runs entire simulation up to n_iter times with current participants
    pub fn run(&mut self) {
        for _ in 0..self.max_iter {
            for j in 0..10 {
                for i in 0..POPULATION_SIZE {
                    self.execute_round_and_update_scores(i, j);
                }
            }
        }
    }

    /// returns the genome of the top [`GENERATION_SIZE`] performing opponents and their scores
    pub fn select_ten_fittest_and_bestscore(&self) -> (Box<[Genome]>, i32) {
        let mut score_acc: Vec<(u8, i32)> = Vec::new();
        for j in 0..10 {
            let organism: u8 = self.opponents[j].strategy_name.parse().unwrap();
            let mut acc = 0;
            for i in 0..POPULATION_SIZE {
                let (score_part, _) = self.scores[(i, j)];
                acc += score_part
            }
            score_acc.push((organism, acc))
        }
        score_acc.sort_by_key(|&(_, n)| n);
        score_acc.reverse();
        let mut leaderboard: Vec<Genome> =
            score_acc.iter().map(|&(c, _)| number_to_genome(c)).collect();
        while leaderboard.len() > 10 {
            let _ = leaderboard.pop();
        }
        let (_, score_of_best) = score_acc[0];
        (leaderboard.into_boxed_slice(), score_of_best)
    }
}

/// Mutates gene by NOT-ing its value at a random index.
pub fn mutate(gene: &mut [bool]) {
    let i = rand::thread_rng().gen_range(0..=4);
    gene[i] = !gene[i];
}

/// Given two parent genomes, returns two child genomes with a 10% chance of mutation.
pub fn reproduce(p1: &Genome, p2: &Genome) -> Genome {
    let mut child = [false; GENOME_LENGTH as usize];
    for idx in 0..GENOME_LENGTH {
        let i = idx as usize;
        if i % 2 == 0 {
            child[i] = p1[i];
        } else {
            child[i] = p2[i];
        }
    }
    let mutation_dist = Bernoulli::new(0.1).unwrap();
    if mutation_dist.sample(&mut rand::thread_rng()) {
        mutate(&mut child);
    }
    Box::new(child)
}

/// Given the fittest old generation of size [GENERATION_SIZE],
/// returns the encoding for the new population, which is a box of encoded genomes
/// of size [POPULATION_SIZE].
pub fn get_new_generation(old_gen: Box<[Genome]>) -> Box<[u8]> {
    let mut new_gen = old_gen.to_vec();
    for i in 0..GENERATION_SIZE {
        let parent1 = &old_gen[i];
        let parent2 = &old_gen[(i + 1) % GENERATION_SIZE];
        let child1 = reproduce(parent1, parent2);
        new_gen.push(child1);
    }
    let new_gen: Vec<u8> = new_gen.iter().map(|g| genome_to_number(g)).collect();
    new_gen.into_boxed_slice()
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

pub type DecisionTable = Box<dyn Fn(Option<Decision>, Option<Decision>) -> Decision>;

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
