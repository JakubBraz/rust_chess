mod moves;

const WIDTH: usize = 8;
const HEIGHT: usize = 8;

#[derive(Debug, Copy, Clone)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Copy, Clone, PartialEq)]
enum Color {
    White, Black
}

struct Pos;

impl Pos {
    const A1: (usize, usize) = (0, 0);
}


#[derive(Copy, Clone)]
struct Piece {
    color: Color,
    kind: PieceType
}

impl Piece {
    fn new(color: Color, kind: PieceType) -> Self {
        Piece { color, kind }
    }
}

struct Board {
    squares: [[Option<Piece>; WIDTH]; HEIGHT],
    move_history: Vec<(Piece, (usize, usize), (usize, usize))>
}

fn new_pieces(color: Color) -> [Option<Piece>; WIDTH] {
    [
        Some(Piece::new(color, PieceType::Rook)),
        Some(Piece::new(color, PieceType::Knight)),
        Some(Piece::new(color, PieceType::Bishop)),
        Some(Piece::new(color, PieceType::Queen)),
        Some(Piece::new(color, PieceType::King)),
        Some(Piece::new(color, PieceType::Bishop)),
        Some(Piece::new(color, PieceType::Knight)),
        Some(Piece::new(color, PieceType::Rook))
    ]
}

fn new_pawns(color: Color) -> [Option<Piece>; WIDTH] {
    [Some(Piece::new(color, PieceType::Pawn)); WIDTH]
}

fn new_empty() -> [Option<Piece>; WIDTH] {
    [None; WIDTH]
}

fn new_board() -> Board {
    Board { squares: [
        new_pieces(Color::White),
        new_pawns(Color::White),
        new_empty(),
        new_empty(),
        new_empty(),
        new_empty(),
        new_pawns(Color::Black),
        new_pieces(Color::Black)
    ],
    move_history: Vec::new()}
}

fn possible_moves(x: usize, y: usize, board: &Board) -> Vec<(usize, usize)> {
    if x < 0 || x >= HEIGHT || y < 0 || y > WIDTH {
        return Vec::new()
    };

    Vec::new()
}

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
