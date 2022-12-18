use super::PieceDefinition;
use super::{Piece, PieceId};
use crate::types::{Player, Bitboard, BDimensions};

// TODO: Remove this
fn null_piece_def(id: PieceId, char_rep: char) -> PieceDefinition {
    PieceDefinition {
        id,
        char_rep,
        is_leader: false,
        can_double_move: false,
        can_castle: false,
        promotion_squares: Bitboard::zero(),
        promo_vals: vec![],
        attack_sliding_deltas: vec![],
        attack_jump_deltas: vec![],
        attack_north: false,
        attack_south: false,
        attack_east: false,
        attack_west: false,
        attack_northeast: false,
        attack_northwest: false,
        attack_southeast: false,
        attack_southwest: false,
        translate_jump_deltas: vec![],
        translate_sliding_deltas: vec![],
        translate_north: false,
        translate_south: false,
        translate_east: false,
        translate_west: false,
        translate_northeast: false,
        translate_northwest: false,
        translate_southeast: false,
        translate_southwest: false,
    }
}


pub struct PieceFactory { }

impl PieceFactory {
    
    // TODO: Remove this
    pub fn make_custom(definition: PieceDefinition, player_num: Player, dims: &BDimensions) -> Piece {
        Piece::new(definition, player_num, dims)
    }
    pub fn make_pawn(id: PieceId, player_num: Player, dims: &BDimensions, promotions: Vec<PieceId>) -> Piece{
        let is_white = player_num == 0;
        let promotion_rank = { if is_white { dims.height - 1 } else { 0 } };
        let mut promotion_squares = Bitboard::zero();
        for i in 0..dims.width {
            promotion_squares.set_bit_at(i, promotion_rank);
        }
        let move_dir = { if is_white { 1 } else { -1 } };
        
        let piece_def = PieceDefinition {
            id,
            char_rep: if is_white { 'P' } else { 'p' },
            is_leader: false,
            can_double_move: true,
            can_castle: false,
            promotion_squares,
            promo_vals: promotions,
            attack_sliding_deltas: vec![],
            attack_jump_deltas: vec![(-1, move_dir), (1, move_dir)],
            attack_north: false,
            attack_south: false,
            attack_east: false,
            attack_west: false,
            attack_northeast: false,
            attack_northwest: false,
            attack_southeast: false,
            attack_southwest: false,
            translate_jump_deltas: vec![(0, move_dir)],
            translate_sliding_deltas: vec![],
            translate_north: false,
            translate_south: false,
            translate_east: false,
            translate_west: false,
            translate_northeast: false,
            translate_northwest: false,
            translate_southeast: false,
            translate_southwest: false,
        };
        
        Piece::new(piece_def, player_num, dims)
    }
    
    
    pub fn make_knight(id: PieceId, player_num: Player, dims: &BDimensions) -> Piece{
        let ch = { if player_num == 0 { 'N' } else { 'n' } };
        Piece::new(null_piece_def(id, ch), player_num, dims)
    }
    pub fn make_king(id: PieceId, player_num: Player, dims: &BDimensions) -> Piece{
        let ch = { if player_num == 0 { 'K' } else { 'k' } };
        Piece::new(null_piece_def(id, ch), player_num, dims)
    }
    pub fn make_rook(id: PieceId, player_num: Player, dims: &BDimensions) -> Piece{
        let ch = { if player_num == 0 { 'R' } else { 'r' } };
        Piece::new(null_piece_def(id, ch), player_num, dims)
    }
    pub fn make_bishop(id: PieceId, player_num: Player, dims: &BDimensions) -> Piece{
        let ch = { if player_num == 0 { 'B' } else { 'b' } };
        Piece::new(null_piece_def(id, ch), player_num, dims)
    }
    pub fn make_queen(id: PieceId, player_num: Player, dims: &BDimensions) -> Piece{
        let ch = { if player_num == 0 { 'Q' } else { 'q' } };
        Piece::new(null_piece_def(id, ch), player_num, dims)
    }
}