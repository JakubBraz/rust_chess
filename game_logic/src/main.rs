use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{current, sleep, spawn};
use std::time::Duration;

use rand::random;
use tungstenite::{accept, HandshakeError, Message, ServerHandshake, WebSocket};
use tungstenite::handshake::server::NoCallback;
use tungstenite::protocol::Role;

use crate::board::{Board, Color, new_board, to_string};
use crate::communication_protocol::{JsonMsg, JsonMsgServer, MsgType, MsgTypeServer};
use crate::game_server::ChannelMsg;

mod moves;
mod board;
mod communication_protocol;
mod game_server;

type BoardsType = HashMap<u32, (Board, Option<u32>, Option<u32>)>;
type ClientsType = HashMap<u32, WebSocket<TcpStream>>;

fn draw_board(board: &Board) {
    let s = to_string(board);
    for c in s.chars() {
        match c {
            '\n' => println!("|"),
            x => print!("|{}", x)
        }
    }
}

// fn thread_game_monitor(boards: Arc<Mutex<HashMap<u32, (Board, Vec<WebSocket<TcpStream>>)>>>) {
fn thread_game_monitor(sender: Sender<ChannelMsg>) {
    loop {
        sender.send(ChannelMsg::ValueMonitor).expect("Cannot send to channel");
        sleep(Duration::from_secs(60));
    }
}

// fn thread_game_controller(rx: Receiver<u64>, mut boards: Arc<HashMap<u64, Board>>) {
//     let number = rx.recv().expect("Cannot receive");
//     boards.insert(number, new_board());
// }

fn try_send(ws: &mut WebSocket<TcpStream>, msg: String) {
    match ws.send(Message::Text(msg)) {
        Ok(_) => log::debug!("Msg sent"),
        Err(e) => log::error!("Cannot send message, error: {}", e)
    }
}

fn broadcast_rooms_message(boards: &BoardsType, clients: &mut ClientsType) {
    log::debug!("Sending rooms to every client");
    let rooms_id: Vec<u32> = boards.iter()
        .filter(|(room_id, (_, white, black))| white.is_some() ^ black.is_some())
        .map(|(&room_id, (_, _, _))| room_id)
        .collect();
    let server_msg = JsonMsgServer { msg_type: MsgTypeServer::Rooms, board: None, rooms: rooms_id, room_id: None, color: None, possible_moves: HashSet::new() };
    let msg = serde_json::to_string(&server_msg).expect("Cannot serialize");

    for ws in clients.values_mut() {
        try_send(ws, msg.clone());
    }
}

fn send_board_user(socket: &mut WebSocket<TcpStream>, board: &Board) {
    let str_board = to_string(board);
    let msg = JsonMsgServer { msg_type: MsgTypeServer::Board, rooms: Vec::new(), board: Some(str_board), room_id: None, color: None, possible_moves: HashSet::new() };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn send_new_room(socket: &mut WebSocket<TcpStream>, room_id: u32, is_white: bool) {
    let msg = JsonMsgServer { msg_type: MsgTypeServer::NewRoom, rooms: Vec::new(), board: None, room_id: Some(room_id), color: Some(if is_white { "white".to_string() } else { "black".to_string() }), possible_moves: HashSet::new() };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn send_board_update(socket: &mut WebSocket<TcpStream>, board: &Board) {
    let msg = JsonMsgServer { msg_type: MsgTypeServer::Board, rooms: Vec::new(), board: Some(to_string(board)), room_id: None, color: None, possible_moves: HashSet::new() };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn send_possible_moves(socket: &mut WebSocket<TcpStream>, moves: HashSet<(usize, usize)>) {
    let msg = JsonMsgServer { msg_type: MsgTypeServer::Possible, rooms: Vec::new(), board: None, room_id: None, color: None, possible_moves: moves };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn send_game_over(socket: &mut WebSocket<TcpStream>, winner: Option<Color>) {
    let result = match winner {
        None => MsgTypeServer::GameResultDraw,
        Some(color) => match color {
            Color::White => MsgTypeServer::GameResultWhiteWon,
            Color::Black => MsgTypeServer::GameResultBlackWon,
        }
    };
    let msg = JsonMsgServer { msg_type: result, rooms: Vec::new(), board: None, room_id: None, color: None, possible_moves: HashSet::new() };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn main() {
    let logger_env = env_logger::Env::default().filter_or("LOG_LEVEL", "TRACE");
    env_logger::Builder::from_env(logger_env).format_timestamp_millis().init();

    let (sender_origin, receiver): (Sender<ChannelMsg>, Receiver<ChannelMsg>) = channel();

    let monitor_sender = sender_origin.clone();
    spawn(|| thread_game_monitor(monitor_sender));
    spawn(|| game_server::handle_game(receiver));

    // let server = TcpListener::bind("127.0.0.1:9977").expect("Cannot create server");
    let server = TcpListener::bind("0.0.0.0:9977").expect("Cannot create server");
    for stream in server.incoming() {
        let sender = sender_origin.clone();
        let tcp_stream = stream.expect("Cannot use tcp stream");
        let tcp_stream_clone = tcp_stream.try_clone().expect("Cannot clone");
        let mut websocket = match accept(tcp_stream) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Cannot create websocket: {}", e);
                continue
            }
        };
        let ws_clone = WebSocket::from_raw_socket(tcp_stream_clone, Role::Server, Some(websocket.get_config().clone()));
        let client_id: u32 = random();

        sender.send(ChannelMsg::NewConnection(client_id, ws_clone)).expect("Cannot send channel");

        let t = spawn(move || {
            let thread_id = current().id();
            log::debug!("New client");
            loop {
                log::debug!("Waiting...");
                let msg = match websocket.read() {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("Cannot read websocket, error: {}", e);
                        log::error!("Sending disconnect to channel and shutting down thread");
                        sender.send(ChannelMsg::Disconnect(client_id)).expect("Cannot send to channel, disconnect");
                        return;
                    }
                };

                log::debug!("{:?} - Received: {:?}", thread_id, msg);
                match msg {
                    Message::Text(m) => {
                        log::debug!("m: {:?}", m);
                        let decoded: JsonMsg = serde_json::from_str(&m).expect("Cannot decode");
                        // let decoded: Value = serde_json::from_str(&m).expect("Cannot decode");
                        sender.send(ChannelMsg::Msg(client_id, decoded)).expect("Cannot send msg channel");
                    }
                    Message::Binary(_) => { log::debug!("binary msg"); }
                    Message::Ping(_) => { log::debug!("ping msg"); }
                    Message::Pong(_) => { log::debug!("pong msg"); }
                    Message::Close(_) => {
                        log::debug!("Closing websocket");
                        sender.send(ChannelMsg::Disconnect(client_id)).expect("Cannot send to channel, disconnect");
                        break;
                    }
                    Message::Frame(_) => { log::debug!("frame msg"); }
                };
            }
        });
    }
}
