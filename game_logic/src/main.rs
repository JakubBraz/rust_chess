use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{current, sleep, spawn};
use std::time::Duration;

use rand::random;
use tungstenite::{accept, Message};

use crate::board::{Board, Color, new_board, to_string};
use crate::communication_protocol::JsonMsg;

mod moves;
mod board;
mod communication_protocol;

fn draw_board(board: &Board) {
    let s = to_string(board);
    for c in s.chars() {
        match c {
            '\n' => println!("|"),
            x => print!("|{}", x)
        }
    }
}

fn thread_game_monitor(boards: Arc<Mutex<HashMap<u32, Board>>>) {
    loop {
        println!("{:?} - {:?}", current().id(), boards.lock().unwrap().keys());
        sleep(Duration::from_secs(10));
    }
}

// fn thread_game_controller(rx: Receiver<u64>, mut boards: Arc<HashMap<u64, Board>>) {
//     let number = rx.recv().expect("Cannot receive");
//     boards.insert(number, new_board());
// }

fn main() {
    let board = new_board();
    draw_board(&board);

    let s = serde_json::to_string(&board).expect("Cannot serialize");
    println!("Serialized: {}", s);

    let mut boards: Arc<Mutex<HashMap<u32, Board>>> = Arc::new(Mutex::new(HashMap::new()));

    let boards_clone = boards.clone();
    spawn(move || thread_game_monitor(boards_clone));

    let server = TcpListener::bind("127.0.0.1:9977").expect("Cannot create server");
    for stream in server.incoming() {
        let boards_clone = boards.clone();
        let t = spawn(move || {
            let thread_id = current().id();
            let s = stream.expect("Cannot use tcp stream");
            let mut websocket = accept(s).expect("Cannot create websocket");
            loop {
                let msg = websocket.read().expect("Cannot read message");

                println!("{:?} - Received: {:?}", thread_id, msg);
                if msg.is_text() {
                    let m = msg.into_text().expect("Cannot decode msg");
                    println!("m: {:?}", m);
                    let decoded: JsonMsg = serde_json::from_str(&m).expect("Cannot decode");
                    // let decoded: Value = serde_json::from_str(&m).expect("Cannot decode");
                    println!("{:?}", decoded);

                    boards_clone.lock().expect("Cannot lock").insert(random(), new_board());
                    websocket.send(Message::Text("ok".into())).expect("Cannot send");
                }
            }
        });
    }
}
