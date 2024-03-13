use crate::board::{Board, Color, new_board, PieceType};

mod moves;
mod board;

fn draw_board(board: &Board) {
    for row in 0 .. 8 {
        for col in 0 .. 8 {
            let icon = match &board.squares[row][col] {
                None => " ",
                Some(p) => match p.kind {
                    PieceType::King => if p.color == Color::White {"K"} else {"k"},
                    PieceType::Queen => if p.color == Color::White {"Q"} else {"q"},
                    PieceType::Rook => if p.color == Color::White {"R"} else {"r"},
                    PieceType::Bishop => if p.color == Color::White {"B"} else {"b"},
                    PieceType::Knight => if p.color == Color::White {"N"} else {"n"},
                    PieceType::Pawn => if p.color == Color::White {"P"} else {"p"},
                }
            };
            print!("|{}", icon);
        }
        println!("|");
    }
}

fn main() {
    println!("Hello, world!");
    // let p = Piece::Pawn {color: 8};
    // println!("This is piece: {:?}", p);
    //
    // let arr: [[u8; 3]; 3] = [
    //     [1, 2, 3],
    //     [10, 20, 30],
    //     [50, 60 , 70]
    // ];
    // println!("This is array: {:?}", arr);
    //
    let board = new_board();

    draw_board(&board);
    //
    // let pp = Pos::A1;

    // let bp = board_position::Position::A1;
    // println!("Board position {:?}", bp);
    // println!("{:?}", bp.coordinates());
}
