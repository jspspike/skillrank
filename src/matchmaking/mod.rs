mod active;

use super::rankings::{Match, Player, Session};
use active::get_active_players;

use std::collections::HashMap;

use rand::prelude::*;
use worker::*;

#[derive(Copy, Clone, Debug, PartialEq)]
struct PlayerInfo {
    id: u16,
    score: u16,
}

#[derive(Clone, Copy)]
pub struct GameInfo {
    games: usize,
    players_per_team: usize,
    stability: f32,
}

pub fn generate_matches(
    players: Vec<u16>,
    ranks: HashMap<u16, Player>,
    session: Session,
    game_info: GameInfo,
) -> Result<Vec<Match>> {
    let mut active_players = get_active_players(players, ranks, session, game_info)?;
    active_players.sort_by(|a, b| b.score.cmp(&a.score));

    for _i in 0..game_info.games {
        let mut team1: Vec<u16> = vec![];
        let mut team2: Vec<u16> = vec![];

        let top_player = active_players.pop().unwrap();
        team1.push(top_player.id);

        let active_player_scores: Vec<u16> =
            active_players.iter().map(|player| player.score).collect();

        let top_player2_probs =
            get_prob(top_player.score, active_player_scores, game_info.stability);
        let rng: f32 = rand::thread_rng().gen_range(0.0..1.0);

        for (index, p) in top_player2_probs.iter().enumerate() {
            if rng < *p {
                team2.push(active_players[index].id);
                break;
            }
        }
    }

    Ok(vec![])
}

fn get_prob(player: u16, others: Vec<u16>, stability: f32) -> Vec<f32> {
    let mut probs: Vec<f32> = vec![];

    let mut total = 0f32;
    others.iter().for_each(|other| {
        let p = (player as isize - *other as isize).abs() as f32;

        probs.push(p);
        total += p;
    });

    let mut prob_total = 0f32;
    probs.iter_mut().for_each(|prob| {
        *prob = 1f32 - (*prob / total as f32);
        *prob = prob.powf(stability);
        prob_total += *prob;
    });

    let mut last = 0f32;
    probs.iter_mut().for_each(|prob| {
        *prob = *prob / prob_total + last;
        last = *prob;
    });

    probs
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
