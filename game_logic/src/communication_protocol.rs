#[derive(Debug, serde::Deserialize)]
pub enum MsgType {
    Join, Create, Move
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonMsg {
    pub msg_type: MsgType,
    pub room_id: Option<u32>,
    pub make_move: Option<((usize, usize), (usize, usize))>
}

#[derive(serde::Serialize)]
pub enum MsgTypeServer {
    Board,
    IllegalMove,
    WhiteWon,
    BlackWon,
    Rooms,
    NewRoom,
}

#[derive(serde::Serialize)]
pub struct JsonMsgServer {
    pub msg_type: MsgTypeServer,
    pub rooms: Vec<u32>,
    pub board: Option<String>,
    pub room_id: Option<u32>,
    pub color: Option<String>,
}
