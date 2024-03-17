#[derive(Debug, serde::Deserialize)]
pub enum MsgType {
    Join, Create, Move
}

#[derive(Debug, serde::Deserialize)]
pub struct JsonMsg {
    msg_type: MsgType,
    room_id: Option<u32>,
    make_move: Option<((usize, usize), (usize, usize))>
}
