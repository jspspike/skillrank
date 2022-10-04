mod active;

use super::rankings::{Match, Player, Session};
use active::get_active_players;

use std::cmp;
use std::collections::{HashMap, VecDeque};

use rand::prelude::*;
use worker::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PlayerInfo {
    id: u16,
    score: i32,
}

#[derive(Clone, Copy)]
pub struct GameInfo {
    pub games: usize,
    pub players_per_team: usize,
    pub stability: f32,
}

pub fn generate_matches(
    players: Vec<u16>,
    ranks: HashMap<u16, Player>,
    session: Session,
    game_info: GameInfo,
) -> Result<Vec<Match>> {
    let mut matches: Vec<Match> = vec![];

    let per_game = game_info.players_per_team * 2;
    let games = cmp::min(players.len() / per_game, game_info.games);
    let total_players = games * per_game;

    let mut active_players =
        VecDeque::from(get_active_players(players, ranks, session, total_players)?);

    for _i in 0..games {
        let mut team1: Vec<u16> = vec![];
        let mut team2: Vec<u16> = vec![];

        let top_player = active_players.pop_front().unwrap();
        team1.push(top_player.id);

        let active_player_scores: Vec<i32> =
            active_players.iter().map(|player| player.score).collect();

        let top_player2 = find_player(
            top_player.score,
            active_player_scores,
            game_info.stability,
            &mut active_players,
        );
        team2.push(top_player2.id);

        let mut overall_diff = top_player.score as isize - top_player2.score as isize;
        let mut overall_score = top_player.score + top_player2.score;

        while team1.len() + team2.len() < per_game {
            let score_avg = overall_score / (team1.len() + team2.len()) as i32;
            let active_player_scores: Vec<i32> =
                active_players.iter().map(|player| player.score).collect();

            let next_player = find_player(
                score_avg,
                active_player_scores,
                game_info.stability,
                &mut active_players,
            );

            let diffs: Vec<i32> = active_players
                .iter()
                .map(|player| (player.score.abs_diff(next_player.score) as i32))
                .collect();
            let next_player2 = find_player(
                overall_diff.abs() as i32,
                diffs,
                game_info.stability,
                &mut active_players,
            );

            if (next_player.score as isize - next_player2.score as isize).is_positive()
                == overall_diff.is_positive()
            {
                team1.push(next_player2.id);
                team2.push(next_player.id);
                overall_diff += next_player2.score as isize - next_player.score as isize;
            } else {
                team1.push(next_player.id);
                team2.push(next_player2.id);
                overall_diff += next_player.score as isize - next_player2.score as isize;
            }

            overall_score += next_player.score;
            overall_score += next_player2.score;
        }

        matches.push(Match {
            id: 0,
            team1,
            team2,
            team1_win: false,
        });
    }

    Ok(matches)
}

fn find_player(
    player_score: i32,
    other_scores: Vec<i32>,
    stability: f32,
    active_players: &mut VecDeque<PlayerInfo>,
) -> PlayerInfo {
    let probs = get_prob(player_score, other_scores, stability);

    let rng_val = rng();
    assert_eq!(rng_val, 0.3);

    let (index, _) = probs
        .iter()
        .enumerate()
        .reduce(|accum, (index, prob)| {
            if rng_val < *prob {
                (index, prob)
            } else {
                accum
            }
        })
        .unwrap();
    active_players.remove(index).unwrap()
}

fn get_prob(player: i32, others: Vec<i32>, stability: f32) -> Vec<f32> {
    let mut probs: Vec<f32> = vec![];

    let mut total = 0.0;
    others.iter().for_each(|other| {
        let p = player.abs_diff(*other) as f32;

        probs.push(p);
        total += p;
    });

    let mut prob_total = 0.0;
    probs.iter_mut().for_each(|prob| {
        *prob = 1.0 - (*prob / total as f32);
        *prob = prob.powf(stability);
        prob_total += *prob;
    });

    let mut last = 0.0;
    probs.iter_mut().for_each(|prob| {
        *prob = *prob / prob_total + last;
        last = *prob;
    });

    probs
}

fn rng() -> f32 {
    let val = vec![2, 3];
    let addr = &val as *const Vec<i32>;

    let rng = addr as u32 % 1000000;
    rng as f32 / 1000000 as f32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_prob() {
        let player = 2000;
        let others = vec![2100, 1400, 1000];

        let probs = get_prob(player, others, 2.0);
        assert_eq!(probs, vec![0.600939, 0.8849765, 0.99999994])
    }
}
