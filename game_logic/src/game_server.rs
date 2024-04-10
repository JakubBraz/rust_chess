use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use rand::random;
use tungstenite::WebSocket;
use crate::{BoardsType, broadcast_rooms_message, send_board_update, send_board_user, send_new_room};
use crate::board::new_board;
use crate::communication_protocol::{JsonMsg, MsgType};
use crate::moves::all_moves;


#[derive(Debug)]
pub enum ChannelMsg {
    NewConnection(u32, WebSocket<TcpStream>),
    Msg(u32, JsonMsg),
    Disconnect(u32),
    ValueMonitor,
}

pub fn handle_game(receiver: Receiver<ChannelMsg>) {
    let mut boards: BoardsType = HashMap::new();
    let mut clients: HashMap<u32, WebSocket<TcpStream>> = HashMap::new();

    loop {
        log::debug!("Waiting for message...");
        let msg = receiver.recv().expect("Cannot receive");
        log::debug!("Msg received {:?}", msg);
        match msg {
            ChannelMsg::NewConnection(websocket_id, websocket) => {
                clients.insert(websocket_id, websocket);
                //todo is this broadcast needed? we need only to send broadcast to this one
                broadcast_rooms_message(&boards, &mut clients);
            }

            ChannelMsg::Msg(websocket_id, decoded) => {
                let mut websocket = clients.get_mut(&websocket_id).expect("Cannot find client");
                match decoded.msg_type {
                    MsgType::Join => {
                        match decoded.room_id {
                            None => log::debug!("No room id provided."),
                            Some(room_id) => {
                                let (board, new_white, new_black) = match boards.get(&room_id) {
                                    None => {
                                        log::debug!("Cannot find room {}", room_id);
                                        (None, None, None)
                                    }
                                    Some((b, white_player, black_player)) => {
                                        match (white_player, black_player) {
                                            (None, Some(black)) => {
                                                send_new_room(websocket, room_id, true);
                                                send_board_user(websocket, &b);
                                                let ws = clients.get_mut(black).expect("Cannot get");
                                                send_board_user(ws, &b);
                                                (Some(b.clone()), Some(websocket_id), Some(black.clone()))
                                            }
                                            (Some(white), None) => {
                                                send_new_room(websocket, room_id, false);
                                                send_board_user(websocket, &b);
                                                let ws = clients.get_mut(white).expect("Cannot get");
                                                send_board_user(ws, &b);
                                                (Some(b.clone()), Some(white.clone()), Some(websocket_id))
                                            }
                                            _ => {
                                                log::warn!("Cannot join full room");
                                                (None, None, None)
                                            }
                                        }
                                    }
                                };
                                match board {
                                    None => { log::debug!("No board found"); }
                                    Some(b) => {
                                        boards.remove(&room_id);
                                        boards.insert(room_id, (b, new_white, new_black));
                                        broadcast_rooms_message(&boards, &mut clients);
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
                            (Some(websocket_id), None)
                        } else {
                            (None, Some(websocket_id))
                        };
                        log::debug!("Broadcasting");
                        log::debug!("trying to insert {:?}", (board_id, (&new_board, white, black)));
                        boards.insert(board_id, (new_board, white, black));
                        log::debug!("broadcast done");

                        broadcast_rooms_message(&boards, &mut clients);
                        let ws = clients.get_mut(&websocket_id).expect("Cannot find");
                        send_new_room(ws, board_id, is_white);
                        log::debug!("Done");
                    }
                    MsgType::Move => {
                        // todo get room id from memory, not from the message
                        let room_id = decoded.room_id.expect("Room id must be provided");
                        let (move_from, move_to) = decoded.make_move.expect("Move must be provided");
                        let is_legal_move = match boards.get(&room_id) {
                            Some((board, Some(white), Some(black))) => {
                                let (white_moves, black_moves) = all_moves(board);
                                if *white == websocket_id && &board.move_history.len() % 2 == 0 {
                                    match white_moves.get(&move_from) {
                                        None => false,
                                        Some(x) => x.contains(&move_to)
                                    }
                                } else if *black == websocket_id && &board.move_history.len() % 2 == 1 {
                                    match black_moves.get(&move_from) {
                                        None => false,
                                        Some(x) => x.contains(&move_to)
                                    }
                                } else {
                                    false
                                }
                            }
                            _ => {
                                false
                            }
                        };

                        if is_legal_move {
                            let (board, white, black) = boards.get_mut(&room_id).expect("Board must be provided");
                            let piece = board.squares[move_from.0][move_from.1].expect("Move must be legal");
                            board.move_history.push((piece, move_from, move_to));
                            board.squares[move_from.0][move_from.1] = None;
                            board.squares[move_to.0][move_to.1] = Some(piece);
                            let client_white = clients.get_mut(&white.expect("Must be provided")).expect("Must be provided");
                            send_board_update(client_white, board);
                            let client_black = clients.get_mut(&black.expect("Must be provided")).expect("Must be provided");
                            send_board_update(client_black, board);
                        }
                        log::debug!("Move done");
                    }
                };
            }

            ChannelMsg::Disconnect(client_id) => {
                log::debug!("Removing client {}", client_id);
                clients.remove(&client_id);
                // todo store board_id in clients instead of searching it
                // todo disconnect both websockets, notify players about game disconnect and game result
                let ids: Vec<u32> = boards.iter()
                    .filter_map(|(&board_id, (_b, white_id, black_id))|
                        if white_id.is_some() && white_id.unwrap() == client_id || black_id.is_some() && black_id.unwrap() == client_id { Some(board_id) } else { None })
                    .collect();
                for id in ids {
                    log::debug!("Removing {}", id);
                    boards.remove(&id);
                }
            }

            ChannelMsg::ValueMonitor => {
                log::info!("Clients: {}", clients.len());
                log::info!("{:?}", clients.keys());
                log::info!("Boards: {}", boards.len());
                for (board_id, (_b, white, black)) in &boards {
                    log::info!("({} - ({:?}, {:?}))", board_id, white, black);
                }
            }
        }
    }
}
