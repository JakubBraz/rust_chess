use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use chess_logic_lib::board::{new_board, to_string, Board, Color, Piece, PieceType, HEIGHT, WIDTH};
use chess_logic_lib::board::Color::{Black, White};
use chess_logic_lib::board::PieceType::{Bishop, King, Knight, Pawn, Queen, Rook};
use chess_logic_lib::moves::allowed_moves;
use neural_network_lib::neural_network::NeuralNetwork;

fn main() {
    // lichess database https://database.lichess.org/
    let mut f = File::open("c:\\Users\\jakubbraz\\Downloads\\lichess_db_standard_rated_2025-08.pgn\\lichess_db_standard_rated_2025-08.pgn")
        .unwrap();
    let reader = BufReader::new(f.try_clone().unwrap());
    let mut iter = reader.lines();
    let mut find_elo = true;

    let mut board = new_board();
    let mut network = NeuralNetwork::new(&[400, 1000, 4]);
    let before_learn_network = network.clone();
    let mut network_target = [0.0; 4];
    let learning_rate = 0.1;

    let t = Instant::now();
    let mut i = 0;
    let duration = Duration::from_secs(3 * 60);

    println!("Learning started, duration: {:?}", duration);
    // for _ in 0..1_000 {
    loop {
        // println!("{i}");
        if i % 10_000 == 0 {
            println!("{i}, elapsed: {:?}", t.elapsed());
        }
        let line = match iter.next() {
            None => break,
            Some(x) => x.unwrap(),
        };
        // println!("{line}");
        if t.elapsed() > duration {
            break;
        }
        i += 1;
        if find_elo && line.starts_with("[WhiteElo") {
            let white_elo = line.as_bytes()[11];
            let next_line = iter.next().unwrap().unwrap();
            let black_elo = next_line.as_bytes()[11];
            if white_elo == b'2' || black_elo == b'2' {
                // println!("{} {}", line, next_line);
                find_elo = false;
            }
        }
        else if !find_elo && line.starts_with("1.") {
            let moves = parse_moves(&line);
            // println!("{:?}", moves);
            let mut network_input = [1.0; 400];
            let mut move_i = 0;
            for mv in moves {
                // println!("move_i {}", move_i);
                let (src, dst) = parse_notation(&board, &mv);
                board.make_move(src, dst);
                // println!("{:?}", (src, dst));
                network_target[0] = (src.0 as f32) / 8.0;
                network_target[1] = (src.1 as f32) / 8.0;
                network_target[2] = (dst.0 as f32) / 8.0;
                network_target[3] = (dst.1 as f32) / 8.0;
                network.training_step(&network_input, &network_target, learning_rate);
                network_input[move_i + 0] = (src.0 as f32) / 8.0;
                network_input[move_i + 1] = (src.1 as f32) / 8.0;
                network_input[move_i + 2] = (dst.0 as f32) / 8.0;
                network_input[move_i + 3] = (dst.1 as f32) / 8.0;
                move_i += 4;
                if move_i == network_input.len() {
                    // todo reconsider stop learning instead of starting over again
                    move_i = 0;
                }
            }
            // println!("{:?}", board.move_history);
            board = new_board();
            find_elo = true;
            f.seek(SeekFrom::Start(0)).unwrap();
            iter = BufReader::new(f.try_clone().unwrap()).lines();
        }

        // println!("{}", line);
    }

    println!("Process done, iterations: {i}, time elapsed {:?}", t.elapsed());
    // fs::write(format!("../neural_networks/chess_network_{i}_{}", chrono::Utc::now().format("%Y_%m_%d_%H_%M_%S")), network.serialize()).unwrap();
    fs::write("../neural_networks/chess_network_test", network.serialize()).unwrap();

    let mut input = [1.0; 400];
    input[0] = 1.0 / 8.0;
    input[1] = 3.0 / 8.0;
    input[2] = 3.0 / 8.0;
    input[3] = 3.0 / 8.0;
    println!("Answer to e4");
    print_test_moves(&before_learn_network, &network, &input);
    println!("Answer to new board");
    let input = [1.0; 400];
    print_test_moves(&before_learn_network, &network, &input);

    // let mut top_elo = 0;
    // let mut i = 0;
    // let mut lines = reader.lines();
    // loop {
    //     let line = match lines.next() {
    //         None => break,
    //         Some(x) => x.unwrap()
    //     };
    //     if line.starts_with("[WhiteElo") {
    //         let next_line = lines.next().unwrap().unwrap();
    //         if line.as_bytes()[11] == b'2' && next_line.as_bytes()[11] == b'2' {
    //             top_elo += 1;
    //         }
    //     }
    //     i += 1;
    //     if i % 1_000_000 == 0 {
    //         println!("line {}, top_elo: {}", i, top_elo);
    //     }
    // }
    // println!("line count: {}", i);
}

fn print_test_moves(before_network: &NeuralNetwork, after_network: &NeuralNetwork, input: &[f32]) {
    let before = before_network.process(&input);
    let after = after_network.process(&input);
    println!("Before learning: {:?}", before);
    println!("After learning: {:?}", after);
    println!("After learning (multiplied): {:?}", after.iter().map(|x| x * 8.0).collect::<Vec<f32>>());
}

fn parse_moves(line: &str) -> Vec<String> {
    line.split("} ")
        .map(|x| x.chars().skip_while(|&x| x != ' ').skip(1).take_while(|&x| x != ' ')
            // .filter(|&x| x != 'x') // ignore capturing
            .collect::<String>())
        .take_while(|x| !x.contains("??")) // don't take anything after a blunder
        .take_while(|x| !x.contains('=') || x.contains("=Q")) // not implemented taking other piece than queen
        .filter(|x| !x.is_empty())
    .collect()
}

fn parse_notation(board: &Board, pgn_move: &str) -> ((usize, usize), (usize, usize)) {
    // println!("move: {}", pgn_move);
    let is_white = board.move_history.len() % 2 == 0;
    let color = if is_white { White } else { Black };
    if pgn_move.starts_with("O-O-O") {
        return if is_white { ((0, 4), (0, 2)) } else { ((7, 4), (7, 2)) };
    }
    if pgn_move.starts_with("O-O") {
        return if is_white { ((0, 4), (0, 6)) } else { ((7, 4), (7, 6)) };
    }
    if pgn_move.contains("=") && !pgn_move.contains("=Q") {
        unreachable!("not implemented");
    }
    if pgn_move.ends_with("??") {
        unreachable!("don't train on blunders");
    }
    let i = pgn_move.chars().enumerate().filter(|&(_i, v)| v.is_ascii_digit()).last().unwrap().0;
    let dst = to_coord(&pgn_move[i - 1 .. i + 1]);

    let first = pgn_move.chars().nth(0).unwrap();
    let piece = if first.is_ascii_uppercase() {
        match first {
            'K' => King,
            'Q' => Queen,
            'R' => Rook,
            'N' => Knight,
            'B' => Bishop,
            _ => unreachable!("unknown piece"),
        }
    }
    else { Pawn };

    let capturing = pgn_move.contains('x');
    let rem = if capturing { &pgn_move[..(i-2)] } else { &pgn_move[..(i-1)] }.as_bytes();

    let src = match rem.len() {
        0 => (None, None),
        1 => if rem[0].is_ascii_lowercase() { (None, Some(to_coord(&format!("{}1", rem[0] as char)).1)) } else { (None, None) },
        2 => if rem[1].is_ascii_lowercase() {
            (None, Some(to_coord(&format!("{}1", rem[1] as char)).1))
        }
        else {
            (Some(to_coord(&format!("a{}", rem[1] as char)).0), None)
        }
        3 => {
            let c = to_coord(&format!("{}{}", rem[1] as char, rem[2] as char));
            (Some(c.0), Some(c.1))
        },
        _ => unreachable!("should not happen"),
    };
    let src = from_finder(board, color, piece, src, dst, capturing);

    (src, dst)
}

fn from_finder(board: &Board, color : Color, piece: PieceType, src: (Option<usize>, Option<usize>), dst: (usize, usize), capturing: bool) -> (usize, usize) {
    // println!("{:?}", (color, piece, src, dst));
    if capturing && board.squares[dst.0][dst.1].is_none() && piece != Pawn { // pawn can capture empty field as en passant
        unreachable!("cant be none, must capture");
    }
    let col_r = if let Some(x) = src.1 { (x, x+1) } else { (0, WIDTH) };
    let row_r = if let Some(x) = src.0 { (x, x+1) } else { (0, HEIGHT) };
    let mut from = (row_r.0..row_r.1).flat_map(|r| (col_r.0..col_r.1).map(move |c| (r, c) ) )
        .filter(|&(r, c)| board.squares[r][c].is_some_and(|x| x.kind == piece && allowed_moves(board, r, c, color).contains(&dst)));
        // .collect();

    let result = from.next().unwrap();

    match from.next() {
        None => result,
        Some(_) => unreachable!("should have only one move"),
    }
}

fn to_coord(square: &str) -> (usize, usize) {
    if square.len() != 2 {
        unreachable!("must have 2 elements, and it is: {}", square);
    }
    let square = square.as_bytes();
    let col = match square[0] as char {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => unreachable!("unknown col")
    };
    let row: u32 = (square[1] as char).to_digit(10).unwrap();
    if row < 1 || row > 8 {
        unreachable!("wrong row value");
    }
    (row as usize - 1, col)
}
