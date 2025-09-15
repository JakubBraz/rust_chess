use std::collections::HashSet;
use crate::board::Color;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum MsgType {
    Join, Create, Move, Possible, Rematch, Ping
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JsonMsg {
    pub msg_type: MsgType,
    pub room_id: u32,
    pub make_move: Option<((usize, usize), (usize, usize))>,
    pub possible_moves: Option<(usize, usize)>,
    pub room_name: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum MsgTypeServer {
    GameResultWhiteWon,
    GameResultBlackWon,
    GameResultDraw,
    NewRoom,
    Possible,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct JsonMsgServer {
    pub msg_type: MsgTypeServer,
    pub board: Option<String>,
    pub room_id: Option<u32>,
    pub color: Option<Color>,
    pub possible_moves: HashSet<(usize, usize)>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum ServerMsg {
    Board{current_board: String, last_move: Option<((usize, usize), (usize, usize))>, in_check: Option<(usize, usize)>},
    Rematch{my_offer: bool},
    Rooms{room_names: Vec<(u32, String)>},
    Disconnected,
    PlayersOnline{count: usize},
}
