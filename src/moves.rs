use std::collections::HashSet;
use crate::{Board, Color, HEIGHT, Piece, PieceType, WIDTH};

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
            Some(p) => match p.color == *current_color {
                true => break,
                false => {
                    result.push((new_row, new_col));
                    break;
                }
            }
        }
        n_row += vec[0];
        n_col += vec[1];
    }
    result
}

// pub fn all_moves() -> [[Option<Vec<(usize, usize)>>; WIDTH]; HEIGHT] {
//     []
// }

pub fn moves(board: &Board, row: usize, col: usize) -> HashSet<(usize, usize)> {
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
                    .filter(|(r, c)| match board.squares[*r][*c] {
                        None => true,
                        Some(new_p) => new_p.color != p.color
                    })
                    .collect()
            }
            PieceType::Pawn => {
                HashSet::new()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use crate::{Board, Color, HEIGHT, new_board, Piece, PieceType, WIDTH};
    use crate::Color::{Black, White};
    use crate::moves::moves;

    fn board_one_piece(row: usize, col: usize, color: Color, kind: PieceType) -> Board {
        let mut board = Board{
            squares: [[None; WIDTH]; HEIGHT],
            move_history: Vec::new()
        };
        board.squares[row][col] = Some(Piece {color, kind});
        board
    }

    #[test]
    fn test_empty_squares() {
        let board = board_one_piece(0, 0, Color::White, PieceType::King);
        let empty_moves = moves(&board, 1, 1);
        assert_eq!(empty_moves, HashSet::new());
    }

    #[test]
    fn test_king_moves() {
        let board = board_one_piece(0, 0, Color::White, PieceType::King);
        let king_moves = moves(&board, 0, 0);
        assert_eq!(king_moves, HashSet::from([(0, 1), (1, 0), (1, 1)]));

        let board = board_one_piece(7, 7, Color::White, PieceType::King);
        let king_moves = moves(&board, 7, 7);
        assert_eq!(king_moves, HashSet::from([(6, 6), (6, 7), (7, 6)]));

        let board = board_one_piece(3, 3, Color::White, PieceType::King);
        let king_moves = moves(&board, 3, 3);
        assert_eq!(king_moves, HashSet::from([(2, 2), (2, 3), (2, 4), (3, 2), (3, 4), (4, 2), (4, 3), (4, 4)]));
    }

    #[test]
    fn test_rook_moves() {
        let board = board_one_piece(0, 0, Color::White, PieceType::Rook);
        let actual_moves = moves(&board, 0, 0);
        assert_eq!(actual_moves, HashSet::from([
            (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0),
            (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7)
        ]));

        let board = new_board();
        let actual_moves = moves(&board, 0, 0);
        assert_eq!(actual_moves, HashSet::new());

        let mut board = new_board();
        board.squares[1][7] = None;
        let actual_moves = moves(&board, 0, 7);
        assert_eq!(actual_moves, HashSet::from([(1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7)]));
    }

    #[test]
    fn test_bishop_moves() {
        let board = board_one_piece(0, 0, Color::White, PieceType::Bishop);
        let actual_moves = moves(&board, 0, 0);
        assert_eq!(actual_moves, HashSet::from([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]));

        let board = board_one_piece(3, 3, Color::White, PieceType::Bishop);
        let actual_moves = moves(&board, 3, 3);
        assert_eq!(actual_moves, HashSet::from([
            (4, 4), (5, 5), (6, 6), (7, 7),
            (2, 4), (1, 5), (0, 6),
            (2, 2), (1, 1), (0, 0),
            (4, 2), (5, 1), (6, 0)
        ]));

        let board = board_one_piece(5, 1, Color::White, PieceType::Bishop);
        let actual_moves = moves(&board, 5, 1);
        assert_eq!(actual_moves, HashSet::from([
            (6, 2), (7, 3),
            (4, 2), (3, 3), (2, 4), (1, 5), (0, 6),
            (4, 0),
            (6, 0)
        ]));

        let board = new_board();
        let actual_moves = moves(&board, 0, 2);
        assert_eq!(actual_moves, HashSet::new());
    }

    #[test]
    fn test_queen_moves() {
        let board = board_one_piece(4, 2, Color::White, PieceType::Queen);
        let actual_moves = moves(&board, 4, 2);
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
        let actual_moves = moves(&board, 0, 3);
        assert_eq!(actual_moves, HashSet::new());

        let mut board = new_board();
        board.squares[1][2] = None;
        board.squares[1][3] = None;
        board.squares[1][4] = None;
        board.squares[4][7] = Some(Piece{color: White, kind: PieceType::Pawn});
        board.squares[3][0] = Some(Piece{color: Black, kind: PieceType::Pawn});
        let actual_moves = moves(&board, 0, 3);
        assert_eq!(actual_moves, HashSet::from([
            (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3),
            (1, 4), (2, 5), (3, 6),
            (1, 2), (2, 1), (3, 0)
        ]));
    }

    #[test]
    fn test_knight_moves() {
        let board = new_board();
        let actual_moves = moves(&board, 0, 1);
        assert_eq!(actual_moves, HashSet::from([(2, 0), (2, 2)]));

        let mut board = board_one_piece(7, 0, Color::White, PieceType::Knight);
        board.squares[5][1] = Some(Piece{color: Black, kind: PieceType::Queen});
        let actual_moves = moves(&board, 7, 0);
        assert_eq!(actual_moves, HashSet::from([(6, 2), (5, 1)]));

        let board = board_one_piece(5, 5, Color::White, PieceType::Knight);
        let actual_moves = moves(&board, 5, 5);
        assert_eq!(actual_moves, HashSet::from([
            (7, 4), (7, 6), (4, 7), (6, 7),
            (3, 4), (3, 6), (4, 3), (6, 3)
        ]));
    }
}
