use std::fs;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn, Thread};
use std::time::Duration;
use chess_logic_lib::board::{new_board, Board, Color, HEIGHT, WIDTH};
use chess_logic_lib::board::Color::{Black, White};
use chess_logic_lib::communication_protocol::{JsonMsg, MsgType, MsgTypeServer, ServerMsg};
use chess_logic_lib::moves::allowed_moves;
use neural_network_lib::neural_network::NeuralNetwork;
use tungstenite::{Message, WebSocket};
use rand::random;
use tungstenite::stream::MaybeTlsStream;

const SERVER_ADDRESS: &str = "ws://127.0.0.1:9977";

fn main() {
    let (tx, rx): (Sender<u8>, Receiver<u8>) = channel();
    let _ = tx.send(0);
    loop {
        println!("waiting for signal...");
        let _ = rx.recv();
        let clone = tx.clone();
        spawn(|| spawn_client(clone));
    }
}

fn spawn_client(tx: Sender<u8>) {
    let (mut socket, resp) = tungstenite::connect(SERVER_ADDRESS).expect("Can't connect to server");
    println!("server response: {:?}", resp);
    let msg = JsonMsg {
        msg_type: MsgType::Create,
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
    let network_name = "chess_network_113932_2025_09_18_11_39_07";
    let network = NeuralNetwork::deserialize(&fs::read_to_string(format!("../neural_networks/{network_name}")).unwrap());
    let mut network_input = [0.0; 400];
    let mut move_i = 0;
    let mut before_first_msg = true;
    loop {
        println!("waiting...");
        match socket.read().unwrap() {
            Message::Text(m) => {
                println!("server msg: {}", m);
                match serde_json::from_str::<ServerMsg>(&m) {
                    Ok(msg) => match msg {
                        ServerMsg::Board { current_board, last_move, in_check } => {
                            if before_first_msg {
                                let _ = tx.send(0);
                                before_first_msg = false;
                            }
                            if last_move.is_none() && my_color == Black {
                                // wait for opponent move
                            }
                            else {
                                if let Some((src, dst)) = last_move {
                                    board.make_move(src, dst);
                                    network_input[move_i + 0] = src.0 as f32;
                                    network_input[move_i + 1] = src.1 as f32;
                                    network_input[move_i + 2] = dst.0 as f32;
                                    network_input[move_i + 3] = dst.1 as f32;
                                    move_i += 4;
                                }

                                let next_move = if move_i == 0 {
                                    let move_d4: bool = random();
                                    if move_d4 {
                                        Some(((1, 3), (3, 3)))
                                    }
                                    else {
                                        Some(((1, 4), (3, 4)))
                                    }
                                }
                                else {
                                    let next_move = neural_network_move(&network, &network_input);
                                    println!("neural network move: {:?}", next_move);
                                    let next_move = if allowed_moves(&board, next_move.0.0, next_move.0.1, my_color).contains(&(next_move.1.0, next_move.1.1)){
                                        Some(next_move)
                                    }
                                    else {
                                        println!("Illegal move, picking random");
                                        pick_random_move(&board, my_color)
                                    };
                                    next_move
                                };

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
                        ServerMsg::Disconnected => break,
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
                                    network_input = [0.0; 400];
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
    println!("Stopping");
}

fn neural_network_move(network: &NeuralNetwork, input: &[f32]) -> ((usize, usize), (usize, usize)) {
    let res = network.process(input);
    println!("neural network; input: {:?}", input);
    println!("output: {:?}", res);
    let res: Vec<usize> = res.iter().map(|&x| x.round() as usize).collect();
    ((res[0], res[1]), (res[2], res[3]))
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
