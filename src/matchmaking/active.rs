use super::{GameInfo, PlayerInfo};
use crate::rankings::{Player, Session};

use std::collections::HashMap;

use worker::*;

pub(super) fn get_active_players(
    players: Vec<u16>,
    ranks: HashMap<u16, Player>,
    session: Session,
    game_info: GameInfo,
) -> Result<Vec<PlayerInfo>> {
    let player_infos = setup_player_info(players, ranks, session)?;
    let mut active_players = find_active_players(player_infos, game_info);
    active_players.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(active_players)
}

fn setup_player_info(
    players: Vec<u16>,
    ranks: HashMap<u16, Player>,
    session: Session,
) -> Result<Vec<Vec<PlayerInfo>>> {
    let mut player_infos: Vec<Vec<PlayerInfo>> = vec![vec![]; session.most_played as usize];

    for player in players {
        let times_played = *session.players.get(&player).unwrap_or(&0);
        let score = match ranks.get(&player) {
            Some(player) => player.score,
            None => return Err(Error::RouteNoDataError),
        };

        player_infos[times_played as usize].push(PlayerInfo { id: player, score });
    }

    Ok(player_infos)
}

fn find_active_players(
    mut player_infos: Vec<Vec<PlayerInfo>>,
    game_info: GameInfo,
) -> Vec<PlayerInfo> {
    let total_players = game_info.players_per_team * 2 * game_info.games;

    let mut players: Vec<PlayerInfo> = vec![];
    let mut times_played_index = 0;
    let mut total_score = 0;

    while players.len() < total_players {
        if player_infos[times_played_index].len() <= total_players - players.len() {
            for p in &player_infos[times_played_index] {
                total_score += p.score;
                players.push(*p);
            }
            times_played_index += 1;
        } else {
            break;
        }
    }

    let avg = total_score as usize / players.len();
    player_infos[times_played_index].sort_by(|a, b| {
        let diffa = (a.score as isize - avg as isize).abs();
        let diffb = (b.score as isize - avg as isize).abs();
        diffa.cmp(&diffb)
    });

    let mut others: Vec<PlayerInfo> = player_infos[times_played_index]
        .drain(0..total_players - players.len())
        .collect();
    players.append(&mut others);

    players
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_active_players() {
        let player_infos: Vec<Vec<PlayerInfo>> = vec![
            vec![
                PlayerInfo { id: 1, score: 2000 },
                PlayerInfo { id: 2, score: 2100 },
                PlayerInfo { id: 3, score: 1900 },
            ],
            vec![
                PlayerInfo { id: 4, score: 1000 },
                PlayerInfo { id: 5, score: 2100 },
                PlayerInfo { id: 6, score: 2600 },
            ],
        ];

        let game_info = GameInfo {
            games: 1,
            players_per_team: 2,
            stability: 2.0,
        };

        let active_players = find_active_players(player_infos, game_info);
        assert_eq!(
            active_players,
            vec![
                PlayerInfo { id: 1, score: 2000 },
                PlayerInfo { id: 2, score: 2100 },
                PlayerInfo { id: 3, score: 1900 },
                PlayerInfo { id: 5, score: 2100 }
            ]
        );
    }
}
