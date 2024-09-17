use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use rand::random;
use tungstenite::WebSocket;
use crate::{BoardsType, broadcast_rooms_message, send_board_update, send_board_user, send_new_room, send_possible_moves};
use crate::board::Color::{Black, White};
use crate::board::{new_board, Color, PieceType};
use crate::board::PieceType::King;
use crate::communication_protocol::{JsonMsg, MsgType};
use crate::moves::allowed_moves;

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
                        let room_id = decoded.room_id;
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
                                // board.color_to_play() == player_color && allowed_moves(board, move_from.0, move_from.1, player_color).contains(&move_to)
                                allowed_moves(board, move_from.0, move_from.1, player_color).contains(&move_to)
                            }
                            _ => false
                        };

                        if is_legal_move {
                            let (board, white, black) = boards.get_mut(&room_id).expect("Board must be provided");
                            let piece = board.squares[move_from.0][move_from.1].expect("Move must be legal");
                            board.move_history.push((piece, move_from, move_to));
                            board.squares[move_from.0][move_from.1] = None;
                            board.squares[move_to.0][move_to.1] = Some(piece);
                            if piece.kind == King {
                                board.king_positions.insert(piece.color, move_to);
                                if move_from.0 == move_to.0 && move_to.1 + 3 == move_from.1 {
                                    board.squares[move_from.0][2] = board.squares[move_from.0][0];
                                    board.squares[move_from.0][0] = None;
                                }
                                else if move_from.0 == move_to.0 && move_from.1 + 2 == move_to.1 {
                                    board.squares[move_from.0][5] = board.squares[move_from.0][7];
                                    board.squares[move_from.0][7] = None;
                                }
                            }
                            let client_white = clients.get_mut(&white.expect("Must be provided")).expect("Must be provided");
                            send_board_update(client_white, board);
                            let client_black = clients.get_mut(&black.expect("Must be provided")).expect("Must be provided");
                            send_board_update(client_black, board);
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
