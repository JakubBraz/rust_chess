use std::net::TcpStream;
use std::thread::{sleep, Thread};
use std::time::Duration;
use chess_logic_lib::board::{new_board, Board, Color, HEIGHT, WIDTH};
use chess_logic_lib::board::Color::{Black, White};
use chess_logic_lib::communication_protocol::{JsonMsg, MsgType, MsgTypeServer, ServerMsg};
use chess_logic_lib::moves::allowed_moves;
use tungstenite::{Message, WebSocket};
use rand::random;
use tungstenite::stream::MaybeTlsStream;

const SERVER_ADDRESS: &str = "ws://127.0.0.1:9977";

fn main() {
    println!("Hello, world! This is AI client");
    spawn_client();
}

fn spawn_client() {
    let (mut socket, resp) = tungstenite::connect(SERVER_ADDRESS).expect("Can't connect to server");
    println!("server response: {:?}", resp);
    let msg = chess_logic_lib::communication_protocol::JsonMsg {
        msg_type: chess_logic_lib::communication_protocol::MsgType::Create,
        room_id: 0,
        make_move: None,
        possible_moves: None,
        room_name: Some("AI room".to_string()),
    };
    println!("msg: {:?}", msg);

    let msg = serde_json::to_string(&msg).unwrap();
    println!("json: {}", msg);

    socket.send(Message::text(msg)).unwrap();

    let mut my_room = 0;
    let mut my_color = White;
    let mut board = new_board();
    loop {
        println!("waiting...");
        match socket.read().unwrap() {
            Message::Text(m) => {
                println!("server msg: {}", m);
                match serde_json::from_str::<ServerMsg>(&m) {
                    Ok(msg) => match msg {
                        ServerMsg::Board { current_board, last_move, in_check } => {
                            if last_move.is_none() && my_color == Black {
                                // wait for opponent move
                            }
                            else {
                                if let Some((from, to)) = last_move {
                                    board.make_move(from, to);
                                }

                                let next_move = pick_random_move(&board, my_color);
                                if next_move.is_some() {
                                    let new_move = JsonMsg {
                                        msg_type: MsgType::Move,
                                        room_id: my_room,
                                        make_move: next_move,
                                        possible_moves: None,
                                        room_name: None,
                                    };
                                    socket.send(
                                        Message::text(serde_json::to_string(&new_move).unwrap())
                                    ).unwrap();
                                }
                            }
                        }
                        ServerMsg::Rematch { .. } => {}
                        ServerMsg::Rooms { .. } => {}
                        ServerMsg::Disconnected => {}
                        ServerMsg::PlayersOnline { .. } => {}
                    }
                    Err(_) => {
                        match serde_json::from_str::<chess_logic_lib::communication_protocol::JsonMsgServer>(&m) {
                            Ok(msg) => match msg.msg_type {
                                MsgTypeServer::GameResultWhiteWon => send_rematch(&mut socket, my_room),
                                MsgTypeServer::GameResultBlackWon => send_rematch(&mut socket, my_room),
                                MsgTypeServer::GameResultDraw => send_rematch(&mut socket, my_room),
                                MsgTypeServer::NewRoom => {
                                    my_room = msg.room_id.unwrap();
                                    my_color = msg.color.unwrap();
                                    board = new_board();
                                }
                                MsgTypeServer::Possible => {}
                            }
                            Err(_) => println!("unknown message"),
                        }
                    }
                }
            }
            Message::Binary(_) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(_) => {}
            Message::Frame(_) => {}
        }
    }
}

fn pick_random_move(board: &Board, color: Color) -> Option<((usize, usize), (usize, usize))> {
    let pieces: Vec<(usize, usize, (usize, usize))> = (0 .. WIDTH)
        .flat_map(|r| (0 .. HEIGHT).map(move |c| (r, c) ))
        .filter(|&(r, c)| board.squares[r][c].is_some_and(|x| x.color == color))
        .flat_map(|(r, c)| allowed_moves(&board, r, c, color).iter().map(|&p| (r, c, p)).collect::<Vec<_>>() )
        .collect();
    if pieces.len() == 0 {
        return None;
    }
    let i: usize = random::<u32>() as usize;
    let (from_r, from_c, (to_r, to_c)) = pieces[i % pieces.len()];
    Some(((from_r, from_c), (to_r, to_c)))
}

fn send_rematch(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, room_id: u32) {
    let msg = JsonMsg {
        msg_type: MsgType::Rematch,
        room_id,
        make_move: None,
        possible_moves: None,
        room_name: None,
    };
    let msg = serde_json::to_string(&msg).unwrap();
    socket.send(Message::Text(msg.into())).unwrap();
}
