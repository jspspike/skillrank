use super::PlayerInfo;
use crate::rankings::{Player, Session};

use std::collections::HashMap;

use worker::*;

pub(super) fn get_active_players(
    players: Vec<u16>,
    ranks: HashMap<u16, Player>,
    session: Session,
    total_players: usize,
) -> Result<Vec<PlayerInfo>> {
    let player_infos = setup_player_info(players, ranks, session)?;
    let mut active_players = find_active_players(player_infos, total_players);
    active_players.sort_by(|a, b| b.score.cmp(&a.score));

    assert_eq!(active_players.len(), total_players);
    Ok(active_players)
}

fn setup_player_info(
    players: Vec<u16>,
    ranks: HashMap<u16, Player>,
    session: Session,
) -> Result<Vec<Vec<PlayerInfo>>> {
    let mut player_infos: Vec<Vec<PlayerInfo>> = vec![vec![]; (session.most_played + 1) as usize];

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
    total_players: usize,
) -> Vec<PlayerInfo> {
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

    if times_played_index >= player_infos.len() {
        return players;
    }

    let avg = if players.len() == 0 {
        10000
    } else {
        total_score / players.len() as i32
    };
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

        let active_players = find_active_players(player_infos, 4);
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

    #[test]
    fn test_find_active_players_no_games() {
        let player_infos: Vec<Vec<PlayerInfo>> = vec![vec![
            PlayerInfo { id: 1, score: 2000 },
            PlayerInfo { id: 2, score: 2150 },
            PlayerInfo { id: 3, score: 1900 },
            PlayerInfo { id: 4, score: 1000 },
            PlayerInfo { id: 5, score: 2100 },
            PlayerInfo { id: 6, score: 2600 },
        ]];

        let active_players = find_active_players(player_infos, 4);
        assert_eq!(
            active_players,
            vec![
                PlayerInfo { id: 6, score: 2600 },
                PlayerInfo { id: 2, score: 2150 },
                PlayerInfo { id: 5, score: 2100 },
                PlayerInfo { id: 1, score: 2000 },
            ]
        );
    }

    #[test]
    fn test_setup_player_info() {
        let players = vec![0, 1, 2, 3, 4, 5];
        let example = Player {
            name: "Test".to_string(),
            score: 2000,
            wins: 0,
            losses: 0,
        };
        let mut ranks = HashMap::new();
        ranks.insert(0, example.clone());
        ranks.insert(1, example.clone());
        ranks.insert(2, example.clone());
        ranks.insert(3, example.clone());
        ranks.insert(4, example.clone());
        ranks.insert(5, example.clone());

        let mut session_players = HashMap::new();
        session_players.insert(0, 0);
        session_players.insert(1, 0);
        session_players.insert(2, 0);
        session_players.insert(3, 0);
        session_players.insert(4, 0);
        session_players.insert(5, 1);

        let session = crate::rankings::Session {
            players: session_players,
            most_played: 1,
        };

        let player_infos = setup_player_info(players, ranks, session).unwrap();
        let expected: Vec<Vec<PlayerInfo>> = vec![
            vec![
                PlayerInfo { id: 0, score: 2000 },
                PlayerInfo { id: 1, score: 2000 },
                PlayerInfo { id: 2, score: 2000 },
                PlayerInfo { id: 3, score: 2000 },
                PlayerInfo { id: 4, score: 2000 },
            ],
            vec![PlayerInfo { id: 5, score: 2000 }],
        ];
        assert_eq!(player_infos, expected)
    }
}
