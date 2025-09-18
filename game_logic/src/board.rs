use std::collections::HashMap;
use serde::{Serializer};
use crate::board::Color::{Black, White};

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Color {
    White, Black
}

impl Color {
    pub fn opposite(&self) -> Color {
        if self == &Color::White {
            Color::Black
        } else {
            Color::White
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GameStatus {
    InProgress,
    Win(Color),
    Draw
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType
}

impl Piece {
    fn new(color: Color, kind: PieceType) -> Self {
        Piece { color, kind }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [[Option<Piece>; WIDTH]; HEIGHT],
    pub move_history: Vec<(Piece, (usize, usize), (usize, usize))>,
    pub position_counter: HashMap<[[Option<Piece>; WIDTH]; HEIGHT], u32>,
    pub max_position_count: u32,
    pub king_positions: HashMap<Color, (usize, usize)>,
    pub game_over: bool,
    pub name: String,
}

impl Board {
    pub fn color_to_play(&self) -> Color {
        if self.move_history.len() % 2 == 0 { White } else { Black }
    }

    pub fn make_move(&mut self, move_from: (usize, usize), move_to: (usize, usize)) {
        let piece = self.squares[move_from.0][move_from.1].unwrap();
        let block_en_passant = self.squares[move_to.0][move_to.1].is_none();
        self.move_history.push((piece, move_from, move_to));
        self.squares[move_from.0][move_from.1] = None;
        self.squares[move_to.0][move_to.1] = Some(piece);
        if piece.kind == PieceType::King {
            self.king_positions.insert(piece.color, move_to);
            if move_from.0 == move_to.0 && move_to.1 + 2 == move_from.1 {
                self.squares[move_from.0][3] = self.squares[move_from.0][0];
                self.squares[move_from.0][0] = None;
            }
            else if move_from.0 == move_to.0 && move_from.1 + 2 == move_to.1 {
                self.squares[move_from.0][5] = self.squares[move_from.0][7];
                self.squares[move_from.0][7] = None;
            }
        }
        else if piece.kind == PieceType::Pawn {
            if move_to.0 == 7 && piece.color == White {
                self.squares[move_to.0][move_to.1] = Some(Piece { color: White, kind: PieceType::Queen });
            }
            else if move_to.0 == 0 && piece.color == Black {
                self.squares[move_to.0][move_to.1] = Some(Piece { color: Black, kind: PieceType::Queen });
            }
            else if move_from.1 + 1 == move_to.1 || move_to.1 + 1 == move_from.1 {
                if self.squares[move_from.0][move_to.1].is_some_and(|x| x.kind == PieceType::Pawn && x.color != piece.color) && block_en_passant {
                    self.squares[move_from.0][move_to.1] = None;
                }
            }
        }

        let val: u32 = *self.position_counter.get(&self.squares).unwrap_or(&0) + 1;
        if self.max_position_count < val {
            self.max_position_count = val;
        }
        self.position_counter.insert(self.squares, val);
    }
}

pub fn to_string(board: &Board) -> String {
    let mut result: String = String::new();
    for row in (0 .. HEIGHT) {
        for col in 0 .. 8 {
            let icon = match &board.squares[row][col] {
                None => ' ',
                Some(p) => match p.kind {
                    PieceType::King => if p.color == Color::White {'K'} else {'k'},
                    PieceType::Queen => if p.color == Color::White {'Q'} else {'q'},
                    PieceType::Rook => if p.color == Color::White {'R'} else {'r'},
                    PieceType::Bishop => if p.color == Color::White {'B'} else {'b'},
                    PieceType::Knight => if p.color == Color::White {'N'} else {'n'},
                    PieceType::Pawn => if p.color == Color::White {'P'} else {'p'},
                }
            };
            result.push(icon);
        }
        result.push('\n');
    }
    let r = &result[0..result.len()-1];
    r.to_string()
}

impl serde::Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(to_string(self).as_str())
    }
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
    let squares = [
        new_pieces(Color::White),
        new_pawns(Color::White),
        new_empty(),
        new_empty(),
        new_empty(),
        new_empty(),
        new_pawns(Color::Black),
        new_pieces(Color::Black)
    ];
    Board {
        squares,
        move_history: Vec::new(),
        king_positions: HashMap::from([(Color::White, (0, 4)), (Color::Black, (7, 4))]),
        game_over: false,
        name: "Room".to_string(),
        position_counter: HashMap::from([(squares, 1)]),
        max_position_count: 1,
    }
}
