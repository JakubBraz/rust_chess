pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    White, Black
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType
}

impl Piece {
    fn new(color: Color, kind: PieceType) -> Self {
        Piece { color, kind }
    }
}

pub struct Board {
    pub squares: [[Option<Piece>; WIDTH]; HEIGHT],
    pub move_history: Vec<(Piece, (usize, usize), (usize, usize))>
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

pub fn new_board() -> Board {
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
