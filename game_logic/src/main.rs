use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{current, sleep, spawn};
use std::time::Duration;

use rand::random;
use tungstenite::{accept, Message, WebSocket};
use tungstenite::protocol::Role;

use crate::board::{Board, Color, new_board, to_string};
use crate::communication_protocol::{JsonMsg, JsonMsgServer, MsgType, MsgTypeServer};

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
        println!("{:?} - {:?}", current().id(), boards.lock().unwrap().keys());
        sleep(Duration::from_secs(10));
    }
}

// fn thread_game_controller(rx: Receiver<u64>, mut boards: Arc<HashMap<u64, Board>>) {
//     let number = rx.recv().expect("Cannot receive");
//     boards.insert(number, new_board());
// }

fn broadcast_rooms_message(boards: &BoardsType, clients: &Arc<Mutex<HashMap<u32, WebSocket<TcpStream>>>>) {
    println!("Sending rooms to every client");
    let rooms_id: Vec<u32> = boards.lock().expect("Cannot lock").iter()
        .filter(|(room_id, (_, white, black))| white.is_some() ^ black.is_some())
        .map(|(&room_id, (_, _, _))| room_id)
        .collect();
    let server_msg = JsonMsgServer {msg_type: MsgTypeServer::Rooms, board: None, rooms: rooms_id, room_id: None, color: None};
    let msg = serde_json::to_string(&server_msg).expect("Cannot serialize");

    for ws in clients.lock().expect("Cannot lock").values_mut() {
        ws.send(Message::Text(msg.clone())).expect("Cannot send");
    }
}

fn send_board_user(socket: &mut WebSocket<TcpStream>, board: &Board) {
    let str_board = to_string(board);
    let msg = JsonMsgServer { msg_type: MsgTypeServer::Board, rooms: Vec::new(), board: Some(str_board), room_id: None, color: None };
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    socket.send(Message::Text(msg)).expect("Cannot send");
}

fn send_new_room(socket: &mut WebSocket<TcpStream>, room_id: u32, is_white: bool) {
    let msg = JsonMsgServer { msg_type: MsgTypeServer::NewRoom, rooms: Vec::new(), board: None, room_id: Some(room_id), color: Some(if is_white {"white".to_string()} else {"black".to_string()})};
    let msg = serde_json::to_string(&msg).expect("Cannot serialize");
    socket.send(Message::Text(msg)).expect("Cannot send");
}

fn main() {
    // todo change Mutex to RwLock
    let mut boards: BoardsType = Arc::new(Mutex::new(HashMap::new()));
    let mut clients: Arc<Mutex<HashMap<u32, WebSocket<TcpStream>>>> = Arc::new(Mutex::new(HashMap::new()));
    // let mut boards: Arc<Mutex<HashMap<u32, Board>>> = Arc::new(Mutex::new(HashMap::new()));

    let clients_clone = clients.clone();
    spawn(move || thread_game_monitor(clients_clone));

    let server = TcpListener::bind("127.0.0.1:9977").expect("Cannot create server");
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
            println!("New client");
            loop {
                println!("Waiting...");
                let msg = websocket.read().expect("Cannot read message");

                println!("{:?} - Received: {:?}", thread_id, msg);
                match msg {
                    Message::Text(m) => {
                        println!("m: {:?}", m);
                        let decoded: JsonMsg = serde_json::from_str(&m).expect("Cannot decode");
                        // let decoded: Value = serde_json::from_str(&m).expect("Cannot decode");
                        println!("{:?}", decoded);

                        match decoded.msg_type {
                            MsgType::Join => {
                                match decoded.room_id {
                                    None => println!("No room id provided."),
                                    Some(room_id) => {
                                        let (board, new_white, new_black) = match boards_clone.lock().expect("Cannot lock").get(&room_id) {
                                            None => {
                                                println!("Cannot find room {}", room_id);
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
                                            None => {println!("No board found");}
                                            Some(b) => {
                                                // println!("join lock remove");
                                                boards_clone.lock().expect("Cannot lock").remove(&room_id);
                                                // println!("join remove");
                                                boards_clone.lock().expect("Cannot lock").insert(room_id, (b, new_white, new_black));
                                                println!("join done");
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
                                println!("is white {}", is_white);
                                let (white, black) = if is_white {
                                    (Some(client_id), None)
                                } else {
                                    (None, Some(client_id))
                                };
                                println!("Broadcasting");
                                println!("trying to insert {:?}", (board_id, (&new_board, white, black)));
                                boards_clone.lock().expect("Cannot lock").insert(board_id, (new_board, white, black));
                                println!("broadcast done");

                                broadcast_rooms_message(&boards_clone, &clients_clone);
                                let mut ws = clients_clone.lock().expect("Cannot lock");
                                let ws = ws.get_mut(&client_id).expect("Cannot find");
                                send_new_room(ws, board_id, is_white);
                                println!("Done");
                            }
                            MsgType::Move => {}
                        };

                        // let server_msg = JsonMsgServer {msg_type: MsgTypeServer::Board, board: Some(to_string(&b)), rooms: Vec::new()};
                        // let msg = serde_json::to_string(&server_msg).expect("Cannot serialize");
                        // websocket.send(Message::Text(msg)).expect("Cannot send");
                    }
                    Message::Binary(_) => {println!("binary msg");}
                    Message::Ping(_) => {println!("ping msg");}
                    Message::Pong(_) => {println!("pong msg");}
                    Message::Close(_) => {
                        println!("Closing websocket");
                        clients_clone.lock().expect("Cannot lock").remove(&client_id);
                        break;
                    }
                    Message::Frame(_) => {println!("frame msg");}
                };
            }
        });
    }
}
