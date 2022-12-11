//Pieces that a player has
use crate::types::{Bitboard, BIndex, Player, BDimensions, Centipawns};
use crate::piece::{Piece, PieceFactory};

use crate::constants::piece_scores::*;

/// Represents a set of pieces for a player
/// custom is a vec of custom piece
#[derive(Clone, Debug)]
pub struct PieceSet {
    pub occupied: Bitboard,
    pub king: Piece,
    pub queen: Piece,
    pub bishop: Piece,
    pub knight: Piece,
    pub rook: Piece,
    pub pawn: Piece,
    pub custom: Vec<Piece>,
    pub player_num: Player
}

impl PieceSet {
    // TODO: Once the hardcoded pieces are removed, remove the BDimensions parameter
    pub fn new(player_num: Player, dims: &BDimensions) -> PieceSet {
        PieceSet {
            occupied: Bitboard::zero(),
            king: PieceFactory::make_king(ID_KING, player_num, dims),
            queen: PieceFactory::make_queen(ID_QUEEN, player_num, dims),
            bishop: PieceFactory::make_bishop(ID_BISHOP, player_num, dims),
            knight: PieceFactory::make_knight(ID_KNIGHT, player_num, dims),
            rook: PieceFactory::make_rook(ID_ROOK, player_num, dims),
            pawn: PieceFactory::make_pawn(ID_PAWN, player_num, dims),
            custom: Vec::new(),
            player_num
        }
    }

    // TODO: Adapt this to use the new PieceSet
    pub fn piece_at(&self, index: BIndex) -> Option<&Piece> {
        if self.king.bitboard.get_bit(index) {
            Some(&self.king)
        } else if self.queen.bitboard.get_bit(index)  {
            Some(&self.queen)
        } else if self.bishop.bitboard.get_bit(index)  {
            Some(&self.bishop)
        } else if self.knight.bitboard.get_bit(index)  {
            Some(&self.knight)
        } else if self.rook.bitboard.get_bit(index)  {
            Some(&self.rook)
        } else if self.pawn.bitboard.get_bit(index)  {
            Some(&self.pawn)
        } else {
            for p in self.custom.iter() {
                if p.bitboard.get_bit(index)  {
                    return Some(p);
                }
            }
            None
        }
    }
    pub fn piece_at_mut(&mut self, index: BIndex) -> Option<&mut Piece> {
        if self.king.bitboard.get_bit(index) {
            Some(&mut self.king)
        } else if self.queen.bitboard.get_bit(index)  {
            Some(&mut self.queen)
        } else if self.bishop.bitboard.get_bit(index)  {
            Some(&mut self.bishop)
        } else if self.knight.bitboard.get_bit(index)  {
            Some(&mut self.knight)
        } else if self.rook.bitboard.get_bit(index)  {
            Some(&mut self.rook)
        } else if self.pawn.bitboard.get_bit(index)  {
            Some(&mut self.pawn)
        } else {
            for p in self.custom.iter_mut() {
                if p.bitboard.get_bit(index)  {
                    return Some(p);
                }
            }
            None
        }
    }

    pub fn get_piece_refs(&self) -> Vec<&Piece> {
        let mut return_vec = Vec::with_capacity(6);
        return_vec.push(&self.king);
        return_vec.push(&self.queen);
        return_vec.push(&self.bishop);
        return_vec.push(&self.knight);
        return_vec.push(&self.rook);
        return_vec.push(&self.pawn);
        for p in &self.custom {
            return_vec.push(p);
        }
        return_vec
    }

    //Recomputes occupied bb
    pub fn update_occupied(&mut self) {
        self.occupied = Bitboard::zero();
        self.occupied |= &self.king.bitboard;
        self.occupied |= &self.queen.bitboard;
        self.occupied |= &self.bishop.bitboard;
        self.occupied |= &self.knight.bitboard;
        self.occupied |= &self.rook.bitboard;
        self.occupied |= &self.pawn.bitboard;
        for p in &self.custom {
            self.occupied |= &p.bitboard;
        }
    }
    
    // Returns the material score of all pieces in the set, and of only the leader pieces
    pub fn get_material_score(&self) -> (Centipawns, Centipawns) {
        let mut score = 0;
        let mut leader_score = 0;
        let king_score = self.king.bitboard.count_ones() as Centipawns * KING_SCORE;
        score += king_score;
        leader_score += king_score;
        score += self.queen.bitboard.count_ones() as Centipawns * QUEEN_SCORE;
        score += self.rook.bitboard.count_ones() as Centipawns * ROOK_SCORE;
        score += self.knight.bitboard.count_ones() as Centipawns * KNIGHT_SCORE;
        score += self.bishop.bitboard.count_ones() as Centipawns * BISHOP_SCORE;
        score += self.pawn.bitboard.count_ones() as Centipawns * PAWN_SCORE;

        for piece in &self.custom {
            let piece_total_score = piece.get_material_score_all();
            score += piece_total_score;
            if piece.is_leader() {
                leader_score += piece_total_score;
            }
        }
        (score, leader_score)
    }
}