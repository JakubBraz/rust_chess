use std::collections::HashSet;

#[derive(Debug, serde::Deserialize)]
pub enum MsgType {
    Join, Create, Move, Possible, Rematch, Ping
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonMsg {
    pub msg_type: MsgType,
    pub room_id: u32,
    pub make_move: Option<((usize, usize), (usize, usize))>,
    pub possible_moves: Option<(usize, usize)>,
    pub room_name: Option<String>,
}

#[derive(serde::Serialize)]
pub enum MsgTypeServer {
    Board,
    GameResultWhiteWon,
    GameResultBlackWon,
    GameResultDraw,
    NewRoom,
    Possible,
}

#[derive(serde::Serialize)]
pub struct JsonMsgServer {
    pub msg_type: MsgTypeServer,
    pub board: Option<String>,
    pub room_id: Option<u32>,
    pub color: Option<String>,
    pub possible_moves: HashSet<(usize, usize)>,
}

#[derive(serde::Serialize)]
pub enum ServerMsg {
    Board{current_board: String, last_move: Option<((usize, usize), (usize, usize))>},
    Rematch{my_offer: bool},
    Rooms{room_names: Vec<(u32, String)>},
    Disconnected,
}
