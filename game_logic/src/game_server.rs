use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use rand::random;
use tungstenite::protocol::Role;
use tungstenite::WebSocket;
use crate::{BoardsType, broadcast_rooms_message, send_board_update, send_new_room, send_possible_moves, send_game_over, send_rematch_offer};
use crate::board::Color::{Black, White};
use crate::board::{new_board, Board, Color, GameStatus, Piece, PieceType};
use crate::board::PieceType::{King, Pawn};
use crate::communication_protocol::{JsonMsg, MsgType};
use crate::moves::{all_potential_attacks, all_potential_moves, allowed_moves, game_result};

#[derive(Debug)]
pub enum ChannelMsg {
    NewConnection(u32, WebSocket<TcpStream>),
    Msg(u32, JsonMsg),
    Disconnect(u32),
    ValueMonitor,
}

pub fn handle_game(receiver: Receiver<ChannelMsg>) {
    let mut boards: BoardsType = HashMap::new();
    //todo move white_id and black_id out of the "boards" variable, set them on JOIN message
    let mut clients: HashMap<u32, WebSocket<TcpStream>> = HashMap::new();

    loop {
        log::debug!("Waiting for message...");
        let msg = receiver.recv().expect("Cannot receive");
        log::debug!("Msg received");
        match msg {
            ChannelMsg::NewConnection(websocket_id, websocket) => {
                let ws_clone = clone_ws(&websocket);
                clients.insert(websocket_id, websocket);
                broadcast_rooms_message(&boards, &mut HashMap::from([(websocket_id, ws_clone)]));
            }

            ChannelMsg::Msg(websocket_id, decoded) => {
                let mut websocket = clone_ws(&clients.get(&websocket_id).expect("Cannot find client"));
                match decoded.msg_type {
                    MsgType::Create => {
                        let board_id: u32 = random();
                        let mut new_board = new_board();
                        match decoded.room_name {
                            None => {}
                            Some(name) => if name.len() < 100 {
                                new_board.name = name;
                            }
                        };
                        // let ws = WebSocket::from_raw_socket(stream_clone, Role::Server, Some(websocket.get_config().clone()));
                        let is_white: bool = random();
                        log::debug!("is white {}", is_white);
                        let (white, black) = if is_white {
                            (Some(websocket_id), None)
                        } else {
                            (None, Some(websocket_id))
                        };
                        boards.insert(board_id, (new_board, white, black));

                        broadcast_rooms_message(&boards, &mut clients);
                        let ws = clients.get_mut(&websocket_id).expect("Cannot find");
                        send_new_room(ws, board_id, is_white);
                        log::debug!("Done");
                    }
                    MsgType::Join => {
                        let room_id = decoded.room_id;
                        let (board, new_white, new_black) = match boards.get(&room_id) {
                            None => {
                                log::debug!("Cannot find room {}", room_id);
                                (None, None, None)
                            }
                            Some((b, white_player, black_player)) => {
                                match (white_player, black_player) {
                                    (None, Some(black)) => {
                                        send_new_room(&mut websocket, room_id, true);
                                        send_board_update(&mut websocket, &b, None);
                                        let ws = clients.get_mut(black).expect("Cannot get");
                                        send_board_update(ws, &b, None);
                                        (Some(b.clone()), Some(websocket_id), Some(black.clone()))
                                    }
                                    (Some(white), None) => {
                                        send_new_room(&mut websocket, room_id, false);
                                        send_board_update(&mut websocket, &b, None);
                                        let ws = clients.get_mut(white).expect("Cannot get");
                                        send_board_update(ws, &b, None);
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
                    MsgType::Rematch => {
                        let room_id = decoded.room_id;
                        let (old_board, white, black) = match boards.get(&room_id) {
                            None => {
                                log::warn!("Cannot find rematch room {}", room_id);
                                return;
                            }
                            Some((board, white, black)) => (board, *white, *black)
                        };

                        if old_board.game_over {
                            match (white, black) {
                                (Some(white), Some(black)) => {
                                    match websocket_id {
                                        x if x == white => {
                                            boards.insert(room_id, (old_board.clone(), None, Some(websocket_id)));
                                            let black_socket = clients.get_mut(&black).expect("Cannot find");
                                            send_rematch_offer(black_socket, false);
                                            send_rematch_offer(&mut websocket, true);
                                        }
                                        x if x == black => {
                                            boards.insert(room_id, (old_board.clone(), Some(websocket_id), None));
                                            let white_socket = clients.get_mut(&white).expect("Cannot find");
                                            send_rematch_offer(white_socket, false);
                                            send_rematch_offer(&mut websocket, true);
                                        }
                                        _ => {
                                            log::warn!("Wrong websocket id");
                                        }
                                    }
                                }
                                (Some(white), None) if white != websocket_id => {
                                    let new_board = new_board();
                                    let white_socket = clients.get_mut(&white).expect("Cannot find");
                                    send_new_room(white_socket, room_id, true);
                                    send_board_update(white_socket, &new_board, None);;
                                    send_new_room(&mut websocket, room_id, false);
                                    send_board_update(&mut websocket, &new_board, None);
                                    boards.insert(room_id, (new_board, Some(white), Some(websocket_id)));
                                }
                                (None, Some(black)) if black != websocket_id => {
                                    let new_board = new_board();
                                    let black_socket = clients.get_mut(&black).expect("Cannot find");
                                    send_new_room(black_socket, room_id, false);
                                    send_board_update(black_socket, &new_board, None);;
                                    send_new_room(&mut websocket, room_id, true);
                                    send_board_update(&mut websocket, &new_board, None);
                                    boards.insert(room_id, (new_board, Some(websocket_id), Some(black)));
                                }
                                _ => {
                                    log::warn!("Unexpected state");
                                }
                            }
                        }
                    }
                    MsgType::Move => {
                        // todo get room id from memory, not from the message
                        let room_id = decoded.room_id;
                        let (move_from, move_to) = decoded.make_move.expect("Move must be provided");
                        let is_legal_move = match boards.get(&room_id) {
                            Some((board, Some(white), Some(black))) => {
                                let player_color = match websocket_id {
                                    x if x == *white => White,
                                    x if x == *black => Black,
                                    x => {
                                        log::warn!("Unexpected websocket id: {}", {x});
                                        White
                                    }
                                };
                                board.color_to_play() == player_color && allowed_moves(board, move_from.0, move_from.1, player_color).contains(&move_to)
                                // allowed_moves(board, move_from.0, move_from.1, player_color).contains(&move_to)
                            }
                            _ => false
                        };

                        if is_legal_move {
                            let (board, white, black) = boards.get_mut(&room_id).expect("Board must be provided");
                            board.make_move(move_from, move_to);
                            let client_white = clients.get(&white.expect("Must be provided")).expect("Must be provided");
                            let client_black = clients.get(&black.expect("Must be provided")).expect("Must be provided");
                            send_board_update(&mut clone_ws(client_white), board, Some((move_from, move_to)));
                            send_board_update(&mut clone_ws(client_black), board, Some((move_from, move_to)));

                            match game_result(board) {
                                GameStatus::InProgress => {
                                    board.game_over = false;
                                }
                                GameStatus::Win(c) => {
                                    send_game_over(&mut clone_ws(client_white), Some(c));
                                    send_game_over(&mut clone_ws(client_black), Some(c));
                                    board.game_over = true;
                                }
                                GameStatus::Draw => {
                                    send_game_over(&mut clone_ws(client_white), None);
                                    send_game_over(&mut clone_ws(client_black), None);
                                    board.game_over = true;
                                }
                            };
                        }
                        log::debug!("Move done");
                    }
                    MsgType::Possible => {
                        match decoded.possible_moves {
                            None => {}
                            Some((row, col)) => {
                                match boards.get(&decoded.room_id) {
                                    Some((board, Some(white_id), Some(black_id))) => {
                                        let my_color = get_player_color(websocket_id, *white_id, *black_id);
                                        let moves = allowed_moves(&board, row, col, my_color);
                                        send_possible_moves(clients.get_mut(&websocket_id).expect("Must be provided"), moves);
                                    }
                                    _ => {}
                                }
                            }
                        };
                    }
                    MsgType::Ping => {
                        log::debug!("Ping message from: {}", websocket_id);
                    }
                };
            }

            ChannelMsg::Disconnect(client_id) => {
                // todo removing elements from hashMap leaves the second player's thread working in the background, eventually a timout closes it manual socket disconnect is needed for resource saving
                // todo or maybe even better, set new server response, like "error", or "game ended", if client receive it, it closes websocket
                // todo this way a consistency would be persevered, only client initiate websocket disconnect
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
                broadcast_rooms_message(&boards, &mut clients);
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

fn get_player_color(websocket_id: u32, white_id: u32, black_id: u32) -> Color {
    if websocket_id == white_id {
        White
    }
    else if websocket_id == black_id {
        Black
    }
    else {
        log::warn!("Cannot find player color, websocket_id: {}", websocket_id);
        White
    }
}

fn clone_ws(websocket: &WebSocket<TcpStream>) -> WebSocket<TcpStream> {
    let tcp_stream: TcpStream = websocket.get_ref().try_clone().unwrap();
    WebSocket::from_raw_socket(tcp_stream, Role::Server, Some(websocket.get_config().clone()))
}
