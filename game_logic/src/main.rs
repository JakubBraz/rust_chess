use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{current, sleep, spawn};
use std::time::Duration;

use rand::random;
use tungstenite::{accept, Message, WebSocket};
use tungstenite::protocol::Role;

use crate::board::{Board, Color, new_board, to_string};
use crate::communication_protocol::{JsonMsg, JsonMsgServer, MsgType, MsgTypeServer};
use crate::moves::all_moves;

mod moves;
mod board;
mod communication_protocol;

type BoardsType = Arc<Mutex<HashMap<u32, (Board, Option<u32>, Option<u32>)>>>;

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
fn thread_game_monitor(boards: Arc<Mutex<HashMap<u32, WebSocket<TcpStream>>>>) {
    loop {
        log::debug!("{:?} - {:?}", current().id(), boards.lock().unwrap().keys());
        sleep(Duration::from_secs(30));
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

fn broadcast_rooms_message(boards: &BoardsType, clients: &Arc<Mutex<HashMap<u32, WebSocket<TcpStream>>>>) {
    log::debug!("Sending rooms to every client");
    let rooms_id: Vec<u32> = boards.lock().expect("Cannot lock").iter()
        .filter(|(room_id, (_, white, black))| white.is_some() ^ black.is_some())
        .map(|(&room_id, (_, _, _))| room_id)
        .collect();
    let server_msg = JsonMsgServer {msg_type: MsgTypeServer::Rooms, board: None, rooms: rooms_id, room_id: None, color: None};
    let msg = serde_json::to_string(&server_msg).expect("Cannot serialize");

    for ws in clients.lock().expect("Cannot lock").values_mut() {
        try_send(ws, msg.clone());
    }
}

fn send_board_user(socket: &mut WebSocket<TcpStream>, board: &Board) {
    let str_board = to_string(board);
    let msg = JsonMsgServer { msg_type: MsgTypeServer::Board, rooms: Vec::new(), board: Some(str_board), room_id: None, color: None };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn send_new_room(socket: &mut WebSocket<TcpStream>, room_id: u32, is_white: bool) {
    let msg = JsonMsgServer { msg_type: MsgTypeServer::NewRoom, rooms: Vec::new(), board: None, room_id: Some(room_id), color: Some(if is_white {"white".to_string()} else {"black".to_string()})};
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn send_board_update(socket: &mut WebSocket<TcpStream>, board: &Board) {
    let msg = JsonMsgServer {msg_type: MsgTypeServer::Board, rooms: Vec::new(), board: Some(to_string(board)), room_id: None, color: None };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    try_send(socket, msg);
}

fn main() {
    let logger_env = env_logger::Env::default().filter_or("LOG_LEVEL", "TRACE");
    env_logger::Builder::from_env(logger_env).format_timestamp_millis().init();

    // todo change Mutex to RwLock
    let mut boards: BoardsType = Arc::new(Mutex::new(HashMap::new()));
    let mut clients: Arc<Mutex<HashMap<u32, WebSocket<TcpStream>>>> = Arc::new(Mutex::new(HashMap::new()));

    let clients_clone = clients.clone();
    spawn(move || thread_game_monitor(clients_clone));

    // let server = TcpListener::bind("127.0.0.1:9977").expect("Cannot create server");
    let server = TcpListener::bind("0.0.0.0:9977").expect("Cannot create server");
    for stream in server.incoming() {
        let boards_clone = boards.clone();
        let tcp_stream = stream.expect("Cannot use tcp stream");
        let tcp_stream_clone = tcp_stream.try_clone().expect("Cannot clone");
        let mut websocket = accept(tcp_stream).expect("Cannot create websocket");
        let client_id: u32 = random();
        clients.lock().expect("Cannot lock").insert(client_id, WebSocket::from_raw_socket(tcp_stream_clone, Role::Server, Some(websocket.get_config().clone())));
        let clients_clone = clients.clone();
        let t = spawn(move || {
            let thread_id = current().id();
            broadcast_rooms_message(&boards_clone, &clients_clone);
            log::debug!("New client");
            loop {
                log::debug!("Waiting...");
                let msg = websocket.read().expect("Cannot read message");

                log::debug!("{:?} - Received: {:?}", thread_id, msg);
                match msg {
                    Message::Text(m) => {
                        log::debug!("m: {:?}", m);
                        let decoded: JsonMsg = serde_json::from_str(&m).expect("Cannot decode");
                        // let decoded: Value = serde_json::from_str(&m).expect("Cannot decode");
                        log::debug!("{:?}", decoded);

                        match decoded.msg_type {
                            MsgType::Join => {
                                match decoded.room_id {
                                    None => log::debug!("No room id provided."),
                                    Some(room_id) => {
                                        let (board, new_white, new_black) = match boards_clone.lock().expect("Cannot lock").get(&room_id) {
                                            None => {
                                                log::debug!("Cannot find room {}", room_id);
                                                (None, None, None)
                                            },
                                            Some((b, white_player, black_player)) => {
                                                match (white_player, black_player) {
                                                    (None, Some(black)) => {
                                                        send_new_room(&mut websocket, room_id, true);
                                                        send_board_user(&mut websocket, &b);
                                                        let mut ws = clients_clone.lock().expect("Cannot lock");
                                                        let ws = ws.get_mut(black).expect("Cannot get");
                                                        send_board_user(ws, &b);
                                                        (Some(b.clone()), Some(client_id), Some(black.clone()))
                                                    }
                                                    (Some(white), None) => {
                                                        send_new_room(&mut websocket, room_id, false);
                                                        send_board_user(&mut websocket, &b);
                                                        let mut ws = clients_clone.lock().expect("Cannot lock");
                                                        let ws = ws.get_mut(white).expect("Cannot get");
                                                        send_board_user(ws, &b);
                                                        (Some(b.clone()), Some(white.clone()), Some(client_id))
                                                    }
                                                    _ => panic!("Unreachable")
                                                }
                                            }
                                        };
                                        match board {
                                            None => {log::debug!("No board found");}
                                            Some(b) => {
                                                // println!("join lock remove");
                                                boards_clone.lock().expect("Cannot lock").remove(&room_id);
                                                // println!("join remove");
                                                boards_clone.lock().expect("Cannot lock").insert(room_id, (b, new_white, new_black));
                                                log::debug!("join done");
                                            }
                                        }
                                    }
                                }
                            }
                            MsgType::Create => {
                                let board_id: u32 = random();
                                let new_board = new_board();
                                // let ws = WebSocket::from_raw_socket(stream_clone, Role::Server, Some(websocket.get_config().clone()));
                                let is_white: bool = random();
                                log::debug!("is white {}", is_white);
                                let (white, black) = if is_white {
                                    (Some(client_id), None)
                                } else {
                                    (None, Some(client_id))
                                };
                                log::debug!("Broadcasting");
                                log::debug!("trying to insert {:?}", (board_id, (&new_board, white, black)));
                                boards_clone.lock().expect("Cannot lock").insert(board_id, (new_board, white, black));
                                log::debug!("broadcast done");

                                broadcast_rooms_message(&boards_clone, &clients_clone);
                                let mut ws = clients_clone.lock().expect("Cannot lock");
                                let ws = ws.get_mut(&client_id).expect("Cannot find");
                                send_new_room(ws, board_id, is_white);
                                log::debug!("Done");
                            }
                            MsgType::Move => {
                                // todo get room id from memory, not from the message
                                let room_id = decoded.room_id.expect("Room id must be provided");
                                let (move_from, move_to) = decoded.make_move.expect("Move must be provided");
                                let is_legal_move = match boards_clone.lock().expect("Cannot lock").get(&room_id) {
                                    Some((board, Some(white), Some(black))) => {
                                        let (white_moves, black_moves) = all_moves(board);
                                        if *white == client_id && &board.move_history.len() % 2 == 0 {
                                            match white_moves.get(&move_from) {
                                                None => false,
                                                Some(x) => x.contains(&move_to)
                                            }
                                        }
                                        else if *black == client_id && &board.move_history.len() % 2 == 1 {
                                            match black_moves.get(&move_from) {
                                                None => false,
                                                Some(x) => x.contains(&move_to)
                                            }
                                        }
                                        else {
                                            false
                                        }
                                    }
                                    _ => {
                                        false
                                    }
                                };

                                if is_legal_move {
                                    let mut board_clients = boards_clone.lock().expect("Cannot lock");
                                    let (board, white, black) = board_clients.get_mut(&room_id).expect("Board must be provided");
                                    let piece = board.squares[move_from.0][move_from.1].expect("Move must be legal");
                                    board.move_history.push((piece, move_from, move_to));
                                    board.squares[move_from.0][move_from.1] = None;
                                    board.squares[move_to.0][move_to.1] = Some(piece);
                                    {
                                        let mut client_white = clients_clone.lock().expect("Cannot lock");
                                        let client_white = client_white.get_mut(&white.expect("Must be provided")).expect("Must be provided");
                                        send_board_update(client_white, board);
                                    }
                                    {
                                        let mut client_black = clients_clone.lock().expect("Cannot lock");
                                        let client_black = client_black.get_mut(&black.expect("Must be provided")).expect("Must be provided");
                                        send_board_update(client_black, board);
                                    }
                                }
                                log::debug!("Move done");
                            }
                        };

                        // let server_msg = JsonMsgServer {msg_type: MsgTypeServer::Board, board: Some(to_string(&b)), rooms: Vec::new()};
                        // let msg = serde_json::to_string(&server_msg).expect("Cannot serialize");
                        // websocket.send(Message::Text(msg)).expect("Cannot send");
                    }
                    Message::Binary(_) => {log::debug!("binary msg");}
                    Message::Ping(_) => {log::debug!("ping msg");}
                    Message::Pong(_) => {log::debug!("pong msg");}
                    Message::Close(_) => {
                        log::debug!("Closing websocket");
                        clients_clone.lock().expect("Cannot lock").remove(&client_id);
                        break;
                    }
                    Message::Frame(_) => {log::debug!("frame msg");}
                };
            }
        });
    }
}
