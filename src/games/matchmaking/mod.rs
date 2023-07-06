mod active;

use crate::rankings::{Match, Player, Session};
use crate::RatingType;
use active::get_active_players;

use std::cmp;
use std::collections::{HashMap, VecDeque};

use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use skillratings::Rating;
use worker::*;

#[derive(Copy, Clone, Debug, PartialEq)]
struct PlayerInfo {
    id: u16,
    rating: RatingType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GameInfo {
    pub games: usize,
    pub players_per_team: usize,
    pub stability: f64,
}

/// First active players are selected, this will be players in the provided session who have played
/// the least number of games and have closest ratings.
///
/// Then the top player will be chosen for team 1, a second player will be chosen for team two at
/// random with a higher probability of being chosen if the player's rating is closest to the player
/// from team 1.
///
/// Then until there are enough players for the game, another player will be chosen for team 1 at
/// random. They will have a higher probability of being chosen if their rating is closer to the
/// average rating of all selected players. Another player will be chosen at random for team 2.
/// They will have a higher probability of being chosen if their rating is closest to the
/// difference in rating between team 1 and team 2.
///
/// Once a game is full it is added and this process repeats until all games are filled.
pub fn generate_matches(
    players: Vec<u16>,
    ranks: &HashMap<u16, Player<RatingType>>,
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

        // Get top player
        let top_player = active_players.pop_front().unwrap();
        team1.push(top_player.id);

        let active_player_scores: Vec<f64> = active_players
            .iter()
            .map(|player| player.rating.rating())
            .collect();

        // Get second player closest to top player's score
        let top_player2 = find_player(
            top_player.rating.rating(),
            active_player_scores,
            game_info.stability,
            &mut active_players,
        );
        team2.push(top_player2.id);

        let mut overall_diff = top_player.rating.rating() - top_player2.rating.rating();
        let mut overall_score = top_player.rating.rating() + top_player2.rating.rating();

        while team1.len() + team2.len() < per_game {
            let score_avg = overall_score / (team1.len() + team2.len()) as f64;
            let active_player_scores: Vec<f64> = active_players
                .iter()
                .map(|player| player.rating.rating())
                .collect();

            // Find next player to team 1 closest to selected players average scores
            let next_player = find_player(
                score_avg,
                active_player_scores,
                game_info.stability,
                &mut active_players,
            );

            // Find player for team 2 where diff(next_player, next_player2) is closest to diff
            // between teams
            let diffs: Vec<f64> = active_players
                .iter()
                .map(|player| (player.rating.rating() - next_player.rating.rating()).abs())
                .collect();
            let next_player2 = find_player(
                overall_diff.abs(),
                diffs,
                game_info.stability,
                &mut active_players,
            );

            if (next_player.rating.rating() - next_player2.rating.rating()).is_sign_positive()
                == overall_diff.is_sign_positive()
            {
                team1.push(next_player2.id);
                team2.push(next_player.id);
                overall_diff += next_player2.rating.rating() - next_player.rating.rating();
            } else {
                team1.push(next_player.id);
                team2.push(next_player2.id);
                overall_diff += next_player.rating.rating() - next_player2.rating.rating();
            }

            overall_score += next_player.rating.rating();
            overall_score += next_player2.rating.rating();
        }

        matches.push(Match {
            id: matches.len() as u16,
            team1,
            team2,
        });
    }

    Ok(matches)
}

fn find_player(
    player_score: f64,
    other_scores: Vec<f64>,
    stability: f64,
    active_players: &mut VecDeque<PlayerInfo>,
) -> PlayerInfo {
    let probs = get_prob(player_score, other_scores, stability);

    let rng_val = rng();
    for (i, prob) in probs.iter().enumerate() {
        if rng_val < *prob {
            return active_players.remove(i).unwrap();
        }
    }

    active_players.pop_back().unwrap()
}

fn get_prob(player: f64, others: Vec<f64>, stability: f64) -> Vec<f64> {
    let mut probs: Vec<f64> = vec![];

    let mut total = 0.0;
    others.iter().for_each(|other| {
        let p = (player - *other).abs();

        probs.push(p.abs());
        total += p;
    });

    let mut prob_total = 0.0;
    probs.iter_mut().for_each(|prob| {
        *prob = 1.0 - (*prob / total);
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

fn rng() -> f64 {
    let mut val: [u8; 1] = [0];
    getrandom(&mut val).unwrap();
    val[0] as f64 / u8::MAX as f64
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_prob() {
        let player = 2000.0;
        let others = vec![1000.0, 1000.0, 2000.0];
        let probs = get_prob(player, others, 1.0);
        assert_eq!(probs, vec![0.25, 0.5, 1.0]);

        let player = 2000.0;
        let others = vec![2100.0, 1400.0, 1000.0];

        let probs = get_prob(player, others, 2.0);
        assert_eq!(probs, vec![0.6009389671361502, 0.8849765258215962, 1.0]);

        let player = 2658.0;
        let others = vec![2344.0, 2638.0, 1986.0];

        let probs = get_prob(player, others, 2.0);
        assert_eq!(probs, vec![0.30645020913647375, 0.9286094600336872, 1.0])
    }

    #[test]
    fn test_get_prob_neg() {
        let player = 50.0;
        let others = vec![
            0.0,
            8.656927044747817,
            91.34307295525218,
            192.00970137063928,
            199.66668579230645,
        ];

        let probs = get_prob(player, others, 1.0);
        assert_eq!(
            probs,
            vec![
                0.22054405366687624,
                0.44618806690254964,
                0.671832080138223,
                0.8381714772910934,
                1.0
            ]
        )
    }

    #[test]
    fn test_get_prob_stability() {
        let player = 2000.0;
        let others = vec![2100.0, 1400.0, 1000.0];

        let probs = get_prob(player, others, 3.0);
        assert_eq!(probs, vec![0.7098786828422877, 0.9405545927209705, 1.0])
    }
}
