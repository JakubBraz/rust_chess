use std::collections::{HashMap, HashSet};
use crate::board::{Board, Color, HEIGHT, PieceType, WIDTH, Piece, GameStatus};
use crate::board::Color::{Black, White};

const START_RANK_WHITE: usize = 1;
const START_RANK_BLACK: usize = 6;

fn move_straight(board: &Board, row: usize, col: usize) -> Vec<(usize, usize)> {
    let current_color = board.squares[row][col].expect("Only occupied squares expected").color;
    let north = move_by_vector(board, &[1, 0], row, col, &current_color);
    let south = move_by_vector(board, &[-1, 0], row, col, &current_color);
    let east = move_by_vector(board, &[0, 1], row, col, &current_color);
    let west = move_by_vector(board, &[0, -1], row, col, &current_color);
    [north, south, east, west].concat()
}

fn move_diagonally(board: &Board, row: usize, col: usize) -> Vec<(usize, usize)> {
    let current_color = board.squares[row][col].expect("Only occupied squares expected").color;
    let north_east = move_by_vector(board, &[1, 1], row, col, &current_color);
    let south_east = move_by_vector(board, &[-1, 1], row, col, &current_color);
    let south_west = move_by_vector(board, &[-1, -1], row, col, &current_color);
    let north_west = move_by_vector(board, &[1, -1], row, col, &current_color);
    [north_east, south_east, south_west, north_west].concat()
}

fn move_by_vector(board: &Board, vec: &[i8; 2], row: usize, col: usize, current_color: &Color) -> Vec<(usize, usize)> {
    let i_row = row as i8;
    let i_col = col as i8;
    let mut result: Vec<(usize, usize)> = Vec::new();
    let mut n_row = i_row + vec[0];
    let mut n_col = i_col + vec[1];
    while n_row >= 0 && n_row < (HEIGHT as i8) && n_col >= 0 && n_col < (WIDTH as i8) {
        let new_row = n_row as usize;
        let new_col = n_col as usize;
        match board.squares[new_row][new_col] {
            None => result.push((new_row, new_col)),
            Some(_) => {
                result.push((new_row, new_col));
                break
            }
        }
        n_row += vec[0];
        n_col += vec[1];
    }
    result
}

pub fn legal_moves(board: &Board, row: usize, col: usize) -> HashSet<(usize, usize)> {
    let moves = potential_moves(board, row, col);
    let current_color = match board.squares[row][col] {
        None => return HashSet::new(),
        Some(piece) => piece.color,
    };
    moves.iter()
        .filter_map(|&(r, c)| {
            match board.squares[r][c] {
                None => Some((r, c)),
                Some(dest_piece) => (dest_piece.color != current_color).then_some((r, c))
            }
        })
        .collect()
}

fn castle_rooks(color: Color) -> ((usize, usize), (usize, usize)) {
    match color {
        White => ((0, 0), (0, 7)),
        Black => ((7, 0), (7, 7)),
    }
}

fn potential_moves(board: &Board, row: usize, col: usize) -> HashSet<(usize, usize)> {
    let i_row = row as i8;
    let i_col = col as i8;
    match board.squares[row][col] {
        None => HashSet::new(),
        Some(p) => match p.kind {
            PieceType::King => {
                let result = [move_straight(board, row, col), move_diagonally(board, row, col)].concat();
                result.into_iter()
                    .map(|(r, c)| (r as i8, c as i8))
                    .filter(|(r, c)| (i_row - r).abs() <= 1 && (i_col - c).abs() <= 1 )
                    .map(|(r, c)| (r as usize, c as usize))
                    .collect()
            }
            PieceType::Queen => [move_straight(board, row, col), move_diagonally(board, row, col)].concat().iter().copied().collect(),
            PieceType::Rook => move_straight(board, row, col).iter().copied().collect(),
            PieceType::Bishop => move_diagonally(board, row, col).iter().copied().collect(),
            PieceType::Knight => {
                let potential_moves = [
                    (i_row + 2, i_col + 1), (i_row + 1, i_col + 2), (i_row - 1, i_col + 2), (i_row - 2, i_col + 1),
                    (i_row - 2, i_col - 1), (i_row - 1, i_col - 2), (i_row + 1, i_col - 2), (i_row + 2, i_col - 1)
                ];
                potential_moves.iter()
                    .filter(|(r, c)| *r >= 0 && *c >= 0 && *r < HEIGHT as i8 && *c < WIDTH as i8)
                    .map(|&(r, c)| (r as usize, c as usize))
                    .collect()
            }
            PieceType::Pawn => {
                let (move_one, move_two, start_rank) = match p.color {
                    Color::White => (i_row + 1, i_row + 2, START_RANK_WHITE),
                    Color::Black => (i_row - 1, i_row - 2, START_RANK_BLACK)
                };
                let moves_forward =
                    if row == start_rank && move_two < HEIGHT as i8 && board.squares[move_one as usize][col].is_none() && board.squares[move_two as usize][col].is_none() {
                        vec!((move_one as usize, col), (move_two as usize, col))
                    }
                    else if move_one < HEIGHT as i8 && board.squares[move_one as usize][col].is_none() {
                        vec!((move_one as usize, col))
                    }
                    else {
                        Vec::new()
                    };
                let moves_capturing: Vec<(usize, usize)> = [(move_one, i_col - 1), (move_one, i_col + 1)].iter()
                    .filter(|(r, c)| *r >= 0 && *c >= 0 && *r < HEIGHT as i8 && *c < WIDTH as i8)
                    .map(|(r, c)| (*r as usize, *c as usize))
                    .filter(|(r, c)| match board.squares[*r][*c] {
                        None => {
                            match board.move_history.last() {
                                None => false,
                                Some(&(last_piece, last_start, last_stop)) =>
                                    last_piece.kind == PieceType::Pawn &&
                                        last_piece.color != p.color &&
                                        (last_start.0 == START_RANK_BLACK || last_start.0 == START_RANK_WHITE) &&
                                        last_start.1 == *c && last_stop.0 == row &&
                                        (last_start.0 + 2 == last_stop.0 || last_stop.0 + 2 == last_start.0)
                            }
                        },
                        Some(_) => true
                    })
                    .collect();

                [moves_forward, moves_capturing].concat().iter().copied().collect()
            }
        }
    }
}

pub fn allowed_moves(board: &Board, row: usize, col: usize, color: Color) -> HashSet<(usize, usize)> {
    let (moves, moving_piece) = match board.squares[row][col] {
        None => (HashSet::new(), PieceType::Pawn),
        Some(piece) =>
            if piece.color != color {
                (HashSet::new(), PieceType::Pawn)
            }
            else {
                match piece.kind {
                    PieceType::King => {
                        let mut moves = legal_moves(board, row, col);
                        let under_attack = &all_potential_attacks(board)[&piece.color.opposite()];
                        if !board.move_history.iter().any(|&(p, _, _)| piece == p) {
                            let (long_castle, short_castle) = castle_rooks(piece.color);
                            if !board.move_history.iter().any(|&(_, from, _)| from == long_castle) {
                                if board.squares[row][col - 1].is_none() &&
                                    board.squares[row][col - 2].is_none() &&
                                    ![(row, col), (row, col - 1)].iter().any(|x| under_attack.contains(x)) {
                                    moves.insert((row, col - 2));
                                }
                            }
                            if !board.move_history.iter().any(|&(_, from, _)| from == short_castle) {
                                if board.squares[row][col + 1].is_none() &&
                                    board.squares[row][col + 2].is_none() &&
                                    ![(row, col), (row, col + 1)].iter().any(|x| under_attack.contains(x)) {
                                    moves.insert((row, col + 2));
                                }
                            }
                        }
                        (moves, piece.kind)
                    },
                    _ => (legal_moves(board, row, col), piece.kind)
                }
            }
    };

    moves.iter().filter(|&&(r, c)| {
        let mut new_board = board.clone();
        new_board.make_move((row, col), (r, c));
        !all_potential_attacks(&new_board)[&color.opposite()].contains(&new_board.king_positions[&color])
    }).copied().collect()
}

fn filter_moves_by_color(board: &Board, occupied_squares: &Vec<(Color, usize, usize)>, to_find: Color, only_attacks: bool) -> HashSet<(usize, usize)> {
    let one_color: Vec<(usize, usize)> = occupied_squares.iter()
        .filter_map(|&(color, r, c)| (color == to_find).then_some((r, c)))
        .collect();
    let attacks: HashSet<(usize, usize)> = one_color.iter()
        .filter(|&&(r, c)| !only_attacks || board.squares[r][c].expect("Must be present").kind != PieceType::Pawn)
        .flat_map(|&(row, col)| potential_moves(board, row, col))
        .collect();
    let pawn_attacks: HashSet<(usize, usize)> = one_color.iter()
        .filter(|&&(r, c)| board.squares[r][c].expect("Must be present").kind == PieceType::Pawn)
        .flat_map(|&(row, col)| {
            let new_row: i8 = match to_find {
                White => row as i8 + 1,
                Black => row as i8 - 1,
            };
            let attacked_squares: HashSet<(usize, usize)> = [(new_row, col as i8 - 1), (new_row, col as i8 + 1)].iter()
                .filter_map(|&(r, c)| (c >= 0 && c < WIDTH as i8).then_some((r as usize, c as usize)) )
                .collect();
            attacked_squares
        })
        .collect();
    &attacks | &pawn_attacks
}

fn all_attacks_moves(board: &Board, only_attacks: bool) -> HashMap<Color, HashSet<(usize, usize)>> {
    let occupied_squares: Vec<(Color, usize, usize)> = (0..HEIGHT).into_iter()
        .flat_map(|r| (0..WIDTH).into_iter().map(move |c| (r, c)))
        .filter(|(r, c)| board.squares[*r][*c].is_some())
        .map(|(r, c)| (board.squares[r][c].unwrap().color, r, c))
        .collect();
    HashMap::from([
        (White, filter_moves_by_color(board, &occupied_squares, White, only_attacks)),
        (Black, filter_moves_by_color(board, &occupied_squares, Black, only_attacks)),
    ])
}

pub fn all_potential_attacks(board: &Board) -> HashMap<Color, HashSet<(usize, usize)>> {
    all_attacks_moves(board, true)
}

pub fn all_potential_moves(board: &Board) -> HashMap<Color, HashSet<(usize, usize)>> {
    all_attacks_moves(board, false)
}

pub fn game_result(board: &Board) -> GameStatus {
    if board.max_position_count == 3 {
       return  GameStatus::Draw;
    }

    let white_result = check_mate(board, &White);
    if white_result == GameStatus::InProgress {
        check_mate(board, &Black)
    }
    else {
        white_result
    }
}

fn check_mate(board: &Board, color: &Color) -> GameStatus {
    let king = board.king_positions[color];
    if all_potential_attacks(board)[&color.opposite()].contains(&king) {
        if (0..HEIGHT)
            .flat_map(|col| (0..WIDTH).map(move |row| (row, col)))
            .filter(|&(r, c)| board.squares[r][c].is_some_and(|x| x.color == *color))
            .flat_map(|(r, c)| allowed_moves(board, r, c, *color).into_iter().map(move |(nr, nc)| ((r, c), (nr, nc))))
            .any(|((move_from), (move_to))| {
                let mut new_board = board.clone();
                new_board.make_move(move_from, move_to);
                !all_potential_attacks(&new_board)[&color.opposite()].contains(&new_board.king_positions[&color])
            }) {
            GameStatus::InProgress
        }
        else {
            GameStatus::Win(color.opposite())
        }
    }
    else if (0..HEIGHT)
        .flat_map(|r| (0..WIDTH).map(move |c| (r, c)))
        .any(|(r, c)| !allowed_moves(board, r, c, *color).is_empty()) {
        GameStatus::InProgress
    }
    else {
        GameStatus::Draw
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};
    use crate::board::{Board, Color, HEIGHT, new_board, Piece, PieceType, WIDTH, GameStatus, to_string};
    use crate::board::PieceType::Pawn;
    use crate::Color::{Black, White};
    use crate::moves::{legal_moves, all_potential_attacks, allowed_moves, all_potential_moves, game_result};

    fn board_one_piece(row: usize, col: usize, color: Color, kind: PieceType) -> Board {
        let mut board = Board{
            squares: [[None; WIDTH]; HEIGHT],
            move_history: Vec::new(),
            king_positions: if kind == PieceType::King { HashMap::from([(color, (row, col))]) } else { HashMap::new() },
            game_over: false,
            name: "Room".to_string(),
            position_counter: HashMap::new(),
            max_position_count: 1,
        };
        board.squares[row][col] = Some(Piece {color, kind});
        board
    }

    #[test]
    fn test_threefold_repetition_draw() {
        let mut board = new_board();
        board.make_move((0, 1), (2, 0));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((7, 1), (5, 0));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((2, 0), (0, 1));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((5, 0), (7, 1));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((0, 1), (2, 0));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((7, 1), (5, 0));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((2, 0), (0, 1));
        assert_eq!(game_result(&board), GameStatus::InProgress);
        board.make_move((5, 0), (7, 1));
        assert_eq!(game_result(&board), GameStatus::Draw);
    }

    #[test]
    fn test_empty_squares() {
        let board = board_one_piece(0, 0, Color::White, PieceType::King);
        let empty_moves = legal_moves(&board, 1, 1);
        assert_eq!(empty_moves, HashSet::new());
    }

    #[test]
    fn test_check_mate() {
        let mut board = board_one_piece(1, 0, White, PieceType::Pawn);
        board.squares[1][2] = Some(Piece {color: White, kind: PieceType::Pawn});
        board.squares[3][1] = Some(Piece {color: White, kind: PieceType::Pawn});
        board.squares[3][7] = Some(Piece {color: White, kind: PieceType::Pawn});
        board.squares[5][0] = Some(Piece {color: Black, kind: PieceType::Pawn});
        board.squares[5][6] = Some(Piece {color: Black, kind: PieceType::Pawn});
        board.squares[6][1] = Some(Piece {color: Black, kind: PieceType::Pawn});
        board.squares[4][3] = Some(Piece {color: Black, kind: PieceType::Rook});
        board.squares[7][5] = Some(Piece {color: Black, kind: PieceType::Rook});
        board.squares[6][6] = Some(Piece {color: Black, kind: PieceType::King});
        board.squares[6][4] = Some(Piece {color: White, kind: PieceType::King});
        board.king_positions = HashMap::from([(Color::White, (6, 4)), (Black, (6, 6))]);
        let result = game_result(&board);
        assert_eq!(result, GameStatus::InProgress);
    }

    #[test]
    fn test_en_passant() {
        let mut board = board_one_piece(3, 4, White, PieceType::Pawn);
        board.squares[0][0] = Some( Piece {color: White, kind: PieceType::King});
        board.squares[7][7] = Some( Piece {color: Black, kind: PieceType::King});
        board.king_positions = HashMap::from([(White, (0, 0)), (Black, (7, 7))]);
        board.move_history.push((Piece { color: White, kind: PieceType::Pawn}, (1, 4), (3, 4)));
        board.squares[3][6] = Some(Piece { color: White, kind: PieceType::Pawn});
        board.squares[3][5] = Some(Piece { color: Black, kind: PieceType::Pawn});
        let moves = allowed_moves(&board, 3, 5, Black);
        assert_eq!(moves, HashSet::from([(2, 5), (2, 4)]));

        board.make_move((3, 5), (2, 4));

        let count = (0 .. HEIGHT).into_iter()
            .flat_map(|c| (0..WIDTH).into_iter().map(move |r| (r, c)))
            .filter(|&(r, c)| board.squares[r][c].is_some())
            .count();
        assert_eq!(count, 4);

        let mut board = board_one_piece(5, 4, White, PieceType::Pawn);
        board.squares[5][3] = Some(Piece { color: Black, kind: PieceType::Pawn});
        board.squares[6][4] = Some(Piece { color: Black, kind: PieceType::Pawn});
        board.squares[6][5] = Some(Piece { color: Black, kind: PieceType::Pawn});
        board.squares[0][0] = Some( Piece {color: White, kind: PieceType::King});
        board.squares[7][7] = Some( Piece {color: Black, kind: PieceType::King});
        board.king_positions = HashMap::from([(White, (0, 0)), (Black, (7, 7))]);
        board.move_history.push((Piece { color: Black, kind: PieceType::Pawn}, (6, 3), (5, 3)));
        let moves = allowed_moves(&board, 5, 4, White);
        assert_eq!(moves, HashSet::from([(6, 5)]));

        let mut board = new_board();
        board.make_move((1, 3), (3, 3));
        board.make_move((6, 4), (4, 4));
        board.make_move((0, 3), (2, 3));
        board.make_move((4, 4), (3, 4));
        board.make_move((1, 7), (2, 7));
        board.make_move((3, 4), (2, 3));
        board.make_move((1, 2), (2, 3));
        board.make_move((6, 2), (4, 2));
        board.make_move((2, 7), (3, 7));
        board.make_move((4, 2), (3, 2));
        board.make_move((3, 7), (4, 7));
        board.make_move((3, 2), (2, 3));
        let pawns = (0..HEIGHT)
            .flat_map(|r| (0..WIDTH).map(move |c| (r, c)))
            .filter(|&(r, c)| board.squares[r][c].is_some_and(|x| x.color == White && x.kind == Pawn))
            .count();
        assert_eq!(pawns, 7);
    }

    #[test]
    fn test_all_potential_moves() {
        let mut board = board_one_piece(2, 1, White, PieceType::Pawn);
        board.squares[3][0] = Some(Piece {color: Black, kind: PieceType::Pawn});
        board.squares[3][2] = Some(Piece {color: Black, kind: PieceType::Pawn});
        let all_moves = all_potential_moves(&board);
        assert_eq!(all_moves[&White], HashSet::from([(3, 0), (3, 1), (3, 2)]));

        let all_attacks = all_potential_attacks(&board);
        assert_eq!(all_attacks[&White], HashSet::from([(3, 0), (3, 2)]));
    }

    #[test]
    fn test_castling() {
        let mut board = board_one_piece(0, 4, Color::White, PieceType::King);
        board.squares[0][0] = Some(Piece {color: White, kind: PieceType::Rook});
        board.squares[0][7] = Some(Piece {color: White, kind: PieceType::Rook});
        let moves = allowed_moves(&board, 0, 4, White);
        assert_eq!(moves.contains(&(0, 2)), true);
        assert_eq!(moves.contains(&(0, 6)), true);

        board.squares[7][5] = Some(Piece {color: Black, kind: PieceType::Rook});
        let moves = allowed_moves(&board, 0, 4, White);
        assert_eq!(moves.contains(&(0, 2)), true);
        assert_eq!(moves.contains(&(0, 6)), false);

        board.squares[7][4] = Some(Piece {color: Black, kind: PieceType::Rook});
        let moves = allowed_moves(&board, 0, 4, White);
        assert_eq!(moves.contains(&(0, 2)), false);
        assert_eq!(moves.contains(&(0, 6)), false);
    }

    #[test]
    fn test_king_check() {
        let mut board = board_one_piece(4, 4, White, PieceType::King);
        board.squares[0][4] = Some(Piece {color: Black, kind: PieceType::Queen});
        board.squares[0][0] = Some(Piece {color: Black, kind: PieceType::Bishop});
        let king = allowed_moves(&board, 4, 4, White);
        assert_eq!(king, HashSet::from([(3, 5), (4, 3), (4, 5), (5, 3)]));
    }

    #[test]
    fn test_king_cannot_move_to_attacked_square() {
        let mut board = board_one_piece(4, 4, White, PieceType::King);
        board.squares[0][3] = Some(Piece{color: Black, kind: PieceType::Rook});
        board.squares[7][5] = Some(Piece{color: Black, kind: PieceType::Rook});
        board.squares[2][4] = Some(Piece{color: Black, kind: PieceType::King});
        let actual_moves = allowed_moves(&board, 4, 4, White);
        assert_eq!(actual_moves, HashSet::from([(5, 4)]));

        let mut board = board_one_piece(4, 4, White, PieceType::King);
        board.move_history.push((Piece {color: White, kind: PieceType::King}, (0, 4), (0, 5)));
        board.squares[6][4] = Some(Piece{color: Black, kind: PieceType::Pawn});
        let all_moves_black = &all_potential_attacks(&board)[&Black];
        let actual_moves = allowed_moves(&board, 4, 4, White);
        assert_eq!(actual_moves, HashSet::from([(3, 3), (3, 4), (3, 5), (4, 3), (4, 5), (5, 4)]));

        let mut board = board_one_piece(4, 4, White, PieceType::King);
        board.squares[5][4] = Some(Piece{color: Black, kind: PieceType::Queen});
        board.squares[7][3] = Some(Piece{color: Black, kind: PieceType::Knight});
        board.squares[3][3] = Some(Piece{color: Black, kind: PieceType::Knight});
        board.squares[4][2] = Some(Piece{color: Black, kind: PieceType::Pawn});
        let all_moves_black = &all_potential_attacks(&board)[&Black];
        let actual_moves = allowed_moves(&board, 4, 4, White);
        assert_eq!(actual_moves, HashSet::from([(3, 5)]));

        let mut board = board_one_piece(3, 1, White, PieceType::King);
        board.squares[5][1] = Some(Piece{color: Black, kind: PieceType::Pawn});
        board.move_history.push((board.squares[3][1].unwrap(), (2, 1), (3, 1)));
        let actual_moves = allowed_moves(&board, 3, 1, White);
        assert_eq!(actual_moves.contains(&(4, 0)), false);
    }

    #[test]
    fn test_king_moves() {
        let board = board_one_piece(0, 0, Color::White, PieceType::King);
        let king_moves = legal_moves(&board, 0, 0);
        assert_eq!(king_moves, HashSet::from([(0, 1), (1, 0), (1, 1)]));

        let board = board_one_piece(7, 7, Color::White, PieceType::King);
        let king_moves = legal_moves(&board, 7, 7);
        assert_eq!(king_moves, HashSet::from([(6, 6), (6, 7), (7, 6)]));

        let board = board_one_piece(3, 3, Color::White, PieceType::King);
        let king_moves = legal_moves(&board, 3, 3);
        assert_eq!(king_moves, HashSet::from([(2, 2), (2, 3), (2, 4), (3, 2), (3, 4), (4, 2), (4, 3), (4, 4)]));
    }

    #[test]
    fn test_rook_moves() {
        let board = board_one_piece(0, 0, Color::White, PieceType::Rook);
        let moves = legal_moves(&board, 0, 0);
        assert_eq!(moves, HashSet::from([
            (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0),
            (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7)
        ]));

        let board = new_board();
        let moves = legal_moves(&board, 0, 0);
        assert_eq!(moves, HashSet::new());

        let mut board = new_board();
        board.squares[1][7] = None;
        let moves = legal_moves(&board, 0, 7);
        assert_eq!(moves, HashSet::from([(1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7)]));
    }

    #[test]
    fn test_bishop_moves() {
        let board = board_one_piece(0, 0, Color::White, PieceType::Bishop);
        let actual_moves = legal_moves(&board, 0, 0);
        assert_eq!(actual_moves, HashSet::from([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]));

        let board = board_one_piece(3, 3, Color::White, PieceType::Bishop);
        let actual_moves = legal_moves(&board, 3, 3);
        assert_eq!(actual_moves, HashSet::from([
            (4, 4), (5, 5), (6, 6), (7, 7),
            (2, 4), (1, 5), (0, 6),
            (2, 2), (1, 1), (0, 0),
            (4, 2), (5, 1), (6, 0)
        ]));

        let board = board_one_piece(5, 1, Color::White, PieceType::Bishop);
        let actual_moves = legal_moves(&board, 5, 1);
        assert_eq!(actual_moves, HashSet::from([
            (6, 2), (7, 3),
            (4, 2), (3, 3), (2, 4), (1, 5), (0, 6),
            (4, 0),
            (6, 0)
        ]));

        let board = new_board();
        let actual_moves = legal_moves(&board, 0, 2);
        assert_eq!(actual_moves, HashSet::new());
    }

    #[test]
    fn test_queen_moves() {
        let board = board_one_piece(4, 2, Color::White, PieceType::Queen);
        let actual_moves = legal_moves(&board, 4, 2);
        assert_eq!(actual_moves, HashSet::from([
            (5, 2), (6, 2), (7, 2),
            (3, 2), (2, 2), (1, 2), (0, 2),
            (4, 3), (4, 4), (4, 5), (4, 6), (4, 7),
            (4, 1), (4, 0),
            (5, 3), (6, 4), (7, 5),
            (3, 3), (2, 4), (1, 5), (0, 6),
            (3, 1), (2, 0),
            (5, 1), (6, 0)
        ]));

        let board = new_board();
        let actual_moves = legal_moves(&board, 0, 3);
        assert_eq!(actual_moves, HashSet::new());

        let mut board = new_board();
        board.squares[1][2] = None;
        board.squares[1][3] = None;
        board.squares[1][4] = None;
        board.squares[4][7] = Some(Piece{color: White, kind: PieceType::Pawn});
        board.squares[3][0] = Some(Piece{color: Black, kind: PieceType::Pawn});
        let actual_moves = legal_moves(&board, 0, 3);
        assert_eq!(actual_moves, HashSet::from([
            (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3),
            (1, 4), (2, 5), (3, 6),
            (1, 2), (2, 1), (3, 0)
        ]));
    }

    #[test]
    fn test_knight_moves() {
        let board = new_board();
        let actual_moves = legal_moves(&board, 0, 1);
        assert_eq!(actual_moves, HashSet::from([(2, 0), (2, 2)]));

        let mut board = board_one_piece(7, 0, Color::White, PieceType::Knight);
        board.squares[5][1] = Some(Piece{color: Black, kind: PieceType::Queen});
        let actual_moves = legal_moves(&board, 7, 0);
        assert_eq!(actual_moves, HashSet::from([(6, 2), (5, 1)]));

        let board = board_one_piece(5, 5, Color::White, PieceType::Knight);
        let actual_moves = legal_moves(&board, 5, 5);
        assert_eq!(actual_moves, HashSet::from([
            (7, 4), (7, 6), (4, 7), (6, 7),
            (3, 4), (3, 6), (4, 3), (6, 3)
        ]));
    }

    #[test]
    fn test_pawn_moves() {
        let board = new_board();
        let actual_moves = legal_moves(&board, 1, 0);
        assert_eq!(actual_moves, HashSet::from([(2, 0), (3, 0)]));

        let actual_moves = legal_moves(&board, 6, 6);
        assert_eq!(actual_moves, HashSet::from([(5, 6), (4, 6)]));

        let board = board_one_piece(6, 1, Color::White, PieceType::Pawn);
        let actual_moves = legal_moves(&board, 6, 1);
        assert_eq!(actual_moves, HashSet::from([(7, 1)]));

        let board = board_one_piece(6, 1, Color::Black, PieceType::Pawn);
        let actual_moves = legal_moves(&board, 6, 1);
        assert_eq!(actual_moves, HashSet::from([(4, 1), (5, 1)]));

        let mut board = board_one_piece(3, 3, Color::Black, PieceType::Pawn);
        board.squares[2][2] = Some(Piece{color: White, kind: PieceType::Pawn});
        board.squares[2][3] = Some(Piece{color: White, kind: PieceType::Pawn});
        board.squares[2][4] = Some(Piece{color: White, kind: PieceType::Pawn});
        let actual_moves = legal_moves(&board, 3, 3);
        assert_eq!(actual_moves, HashSet::from([(2, 2), (2, 4)]));

        let mut board = board_one_piece(4, 4, Color::White, PieceType::Pawn);
        board.squares[4][3] = Some(Piece{color: Black, kind: PieceType::Pawn});
        board.move_history = vec![(Piece{color: Black, kind: PieceType::Pawn}, (6, 3), (4, 3))];
        let actual_moves = legal_moves(&board, 4, 4);
        assert_eq!(actual_moves, HashSet::from([(5, 4), (5, 3)]));
    }
}
