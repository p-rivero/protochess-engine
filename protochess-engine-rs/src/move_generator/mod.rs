use crate::piece::Piece;
use crate::types::*;
use crate::position::Position;
use crate::move_generator::attack_tables::AttackTables;
use crate::move_generator::bitboard_moves::BitboardMoves;
use crate::utils::{from_index, to_index};

pub mod attack_tables;
pub mod bitboard_moves;


lazy_static! {
    static ref ATTACK_TABLES: AttackTables = AttackTables::new();
}


#[derive(Clone, Debug)]
pub struct MoveGen;

impl MoveGen {
    pub fn attack_tables() -> &'static AttackTables {
        &ATTACK_TABLES
    }
    
    pub fn get_legal_moves(position: &mut Position) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        for mv in MoveGen::get_pseudo_moves(position) {
            if !MoveGen::is_move_legal(&mv, position) {
                continue;
            }
            legal_moves.push(mv);
        }
        legal_moves
    }

    /// Iterator that yields pseudo-legal moves from a positon
    pub fn get_pseudo_moves(position: &mut Position) -> impl Iterator<Item=Move> {
        let my_pieces = &position.pieces[position.whos_turn as usize];

        let mut out_bb_moves: Vec<BitboardMoves> = Vec::with_capacity(50);
        let mut out_moves = Vec::with_capacity(50);

        let enemies = &position.occupied & !my_pieces.get_occupied();
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;

        for p in my_pieces.iter() {
            p.output_moves(position, &enemies, &occ_or_not_in_bounds, &mut out_bb_moves, &mut out_moves);
        }
        out_bb_moves.into_iter().flatten().chain(out_moves.into_iter())
    }
    
    ///Iterator that yields only pseudo-legal capture moves
    pub fn get_capture_moves(position: &mut Position) -> impl Iterator<Item=Move> {
        let my_pieces = &position.pieces[position.whos_turn as usize];

        let mut out_bb_moves: Vec<BitboardMoves> = Vec::with_capacity(50);
        let mut out_moves = Vec::with_capacity(50);

        let enemies = &position.occupied & !my_pieces.get_occupied();
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;

        for p in my_pieces.iter() {
            p.output_captures(position, &enemies, &occ_or_not_in_bounds, &mut out_bb_moves, &mut out_moves);
        }
        out_bb_moves.into_iter().flatten().chain(out_moves.into_iter())
    }

    /// Checks if the player to move is in check
    pub fn in_check(position: &mut Position) -> bool {
        let my_pieces = &position.pieces[position.whos_turn as usize];
        let my_leader = my_pieces.get_leader();
        if my_leader.get_num_pieces() > 1 {
            // There are multiple leaders, so the position cannot be in check
            return false;
        }
        MoveGen::index_in_check(my_leader.get_first_index().unwrap(), position)
    }

    /// Checks if a move is legal
    pub fn is_move_legal(mv: &Move, position: &mut Position) -> bool {
        // If the move is castling, check extra conditions
        if mv.get_move_type() == MoveType::KingsideCastle || mv.get_move_type() == MoveType::QueensideCastle {
            // Cannot castle while in check
            if MoveGen::in_check(position) {
                return false;
            }
            let step_index = { if mv.get_move_type() == MoveType::KingsideCastle { mv.get_from()+1 } else { mv.get_from()-1 } };
            // Cannot step through check
            if MoveGen::index_in_check(step_index, position) {
                return false;
            }
        }
        
        // Try the move and skip a turn, then see if we are in check
        position.make_move(*mv, false);
        position.make_move(Move::null(), false);
        // See if we are in check or an explosion has killed the last leader
        // However, if the move causes us to capture the last enemy leader, the move is legal (even if it leaves us in check)
        let legal = !position.leader_is_captured() && (position.enemy_leader_is_captured() || !MoveGen::in_check(position));
        position.unmake_move();
        position.unmake_move();
        legal
    }


    ///Returns the number of legal moves for a position
    pub fn count_legal_moves(position: &mut Position) -> u64{
        let mut nodes = 0u64;
        for mv in MoveGen::get_pseudo_moves(position) {
            if !MoveGen::is_move_legal(&mv, position) {
                continue;
            }
            nodes += 1;
        }
        nodes
    }
    
    
    /// Checks if a given square is attacked by the enemy
    fn index_in_check(index: BIndex, position: &mut Position) -> bool {
        // Get the leader of the player to move
        let enemy = 1 - position.whos_turn;
        let enemy_pieces = &position.pieces[enemy as usize];
        let enemy_occupied = enemy_pieces.get_occupied();
        let inverse_attack = enemy_pieces.get_inverse_attack();
        // Use inverse attack pattern to get the squares that can potentially attack the square
        let attack_tables = MoveGen::attack_tables();
        let occ_or_not_in_bounds = &position.occupied | !&position.dimensions.bounds;
        
        let mut slides = attack_tables.get_sliding_moves_bb(
            index,
            &occ_or_not_in_bounds,
            inverse_attack.attack_north,
            inverse_attack.attack_east,
            inverse_attack.attack_south,
            inverse_attack.attack_west,
            inverse_attack.attack_northeast,
            inverse_attack.attack_northwest,
            inverse_attack.attack_southeast,
            inverse_attack.attack_southwest
        );
        slides &= enemy_occupied;
        while let Some(enemy_piece_index) = slides.lowest_one() {
            let enemy_piece = enemy_pieces.piece_at(enemy_piece_index).unwrap();
            // Found an enemy piece that might attack the last leader
            if MoveGen::slide_targets_index(enemy_piece, enemy_piece_index, index, &occ_or_not_in_bounds) && enemy_piece.attacking_is_legal() {
                return true;
            }
            slides.clear_bit(enemy_piece_index);
        }
        
        // Check jump attacks
        let (x, y) = from_index(index);
        for (dx, dy) in &inverse_attack.attack_jump_deltas {
            let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
            if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                continue;
            }
            let enemy_piece_index = to_index(x2 as BCoord, y2 as BCoord);
            if !enemy_occupied.get_bit(enemy_piece_index) {
                continue;
            }
            // Found an enemy piece that might attack the last leader
            let enemy_piece = enemy_pieces.piece_at(enemy_piece_index).unwrap();
            if enemy_piece.get_movement().attack_jump_deltas.contains(&(-dx, -dy)) && enemy_piece.attacking_is_legal() {
                return true;
            }
        }
        
        // Check sliding deltas
        for run in &inverse_attack.attack_sliding_deltas {
            for (dx, dy) in run {
    
                let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                    break;
                }
    
                let to = to_index(x2 as BCoord, y2 as BCoord);
                //Out of bounds, next sliding moves can be ignored
                if !position.in_bounds(x2 as BCoord, y2 as BCoord) {
                    break;
                }
                if enemy_occupied.get_bit(to) {
                    // Found an enemy piece that might attack the last leader
                    let enemy_piece = enemy_pieces.piece_at(to).unwrap();
                    if MoveGen::sliding_delta_targets_index(enemy_piece, to, index, &occ_or_not_in_bounds) && enemy_piece.attacking_is_legal() {
                        return true;
                    }
                    break;
                }
                //Occupied by own team
                if position.occupied.get_bit(to) {
                    break;
                }
            }
        }
        false
    }
    fn slide_targets_index(piece: &Piece, piece_index: BIndex, target_index: BIndex, occ_or_not_in_bounds: &Bitboard) -> bool {
        let attack_tables = MoveGen::attack_tables();
        let piece_movement = piece.get_movement();
        let slides = attack_tables.get_sliding_moves_bb(
            piece_index,
            occ_or_not_in_bounds,
            piece_movement.attack_north,
            piece_movement.attack_east,
            piece_movement.attack_south,
            piece_movement.attack_west,
            piece_movement.attack_northeast,
            piece_movement.attack_northwest,
            piece_movement.attack_southeast,
            piece_movement.attack_southwest
        );
        slides.get_bit(target_index)
    }
    fn sliding_delta_targets_index(piece: &Piece, piece_index: BIndex, target_index: BIndex, occ_or_not_in_bounds: &Bitboard) -> bool {
        let piece_movement = piece.get_movement();
        let (x, y) = from_index(piece_index);
        for run in &piece_movement.attack_sliding_deltas {
            for (dx, dy) in run {
                let (x2, y2) = (x as i8 + *dx, y as i8 + *dy);
                if x2 < 0 || y2 < 0 || x2 > 15 || y2 > 15 {
                    break;
                }
                let to = to_index(x2 as BCoord, y2 as BCoord);
                if to == target_index {
                    return true;
                }
                // Occupied or out of bounds
                if occ_or_not_in_bounds.get_bit(to) {
                    break;
                }
            }
        }
        false
    }
}
