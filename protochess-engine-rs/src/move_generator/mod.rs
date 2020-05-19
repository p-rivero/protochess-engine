use crate::types::{Dimensions, bitboard, PieceType};
use crate::types::{Move, LineAttackType, AttackDirection };
use crate::types::bitboard::{Bitboard, to_string};
use crate::position::Position;
use crate::position::piece_set::PieceSet;
use crate::move_generator::attack_tables::AttackTables;
use crate::move_generator::bitboard_moves::BitboardMoves;

mod attack_tables;
mod bitboard_moves;
//Iterator that yields possible moves from a position
pub(crate) struct MoveGenerator {
    attack_tables: AttackTables,
}
impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator{
            attack_tables: AttackTables::new(),
        }
    }

    //Generates pseudo-legal moves
    pub fn get_moves(&self, position:&mut Position) -> Vec<Move> {
        let mut return_vec = Vec::with_capacity(250);
        let pseudo = self.get_pseudo_moves(position);
        for m in pseudo {
            if self.is_move_legal(&m, position){
                return_vec.push(m);
            }
        }
        return_vec
    }

    pub fn get_pseudo_moves(&self, position:&Position) -> Vec<Move> {
        let my_pieces: &PieceSet = &position.pieces[position.whos_turn as usize];
        let enemies = &position.occupied & !&my_pieces.occupied;

        //create a vector of iterators
        let mut movelist:Vec<Move> = Vec::with_capacity(250);


        let mut apply_to_each = |mut pieceset:Bitboard, func: fn(&AttackTables, u8, &Bitboard, &Bitboard)-> Bitboard| {
            while !pieceset.is_zero() {
                let index = pieceset.lowest_one().unwrap() as u8;
                let mut raw_attacks = func(&self.attack_tables,index, &position.occupied, &enemies);
                //Do not attack ourselves
                raw_attacks &= !&my_pieces.occupied;
                //Keep only in bounds
                raw_attacks &= &position.bounds;

                while !raw_attacks.is_zero() {
                    let to = raw_attacks.lowest_one().unwrap();
                    raw_attacks.set_bit(to, false);
                    if enemies.bit(to).unwrap() {
                        movelist.push(Move::new(index, to as u8,true));
                    }else{
                        movelist.push(Move::new(index, to as u8,false));
                    }
                }

                pieceset.set_bit(index as usize, false);
            }
        };

        apply_to_each((&my_pieces.king).to_owned(), AttackTables::get_king_attack);
        apply_to_each((&my_pieces.queen).to_owned(), AttackTables::get_queen_attack);
        apply_to_each((&my_pieces.rook).to_owned(), AttackTables::get_rook_attack);
        apply_to_each((&my_pieces.bishop).to_owned(), AttackTables::get_bishop_attack);
        apply_to_each((&my_pieces.knight).to_owned(), AttackTables::get_knight_attack);
        if position.whos_turn == 0 {
            apply_to_each((&my_pieces.pawn).to_owned(), AttackTables::get_north_pawn_attack);
        }else{
            apply_to_each((&my_pieces.pawn).to_owned(), AttackTables::get_south_pawn_attack);
        }

        movelist
    }

    //Checks if a move is legal
    pub fn is_move_legal(&self, move_:&Move, position:&mut Position) -> bool{
        let my_player_num = position.whos_turn;
        position.make_move(move_.to_owned());
        let my_pieces: &PieceSet = &position.pieces[my_player_num as usize];
        let enemies = &position.occupied & !&my_pieces.occupied;

        //Calculate enemies piece sets
        let enemy_pieces: &PieceSet = &position.pieces[position.whos_turn as usize];
        //TODO generalize for >2 players
        let enemy_pawns = &enemy_pieces.pawn;
        let enemy_knights = &enemy_pieces.knight;
        let enemy_bishops = &enemy_pieces.bishop;
        let enemy_queens = &enemy_pieces.queen;
        let enemy_rooks = &enemy_pieces.rook;
        let enemy_kings = &enemy_pieces.king;

        let loc_index = my_pieces.king.lowest_one().unwrap() as u8;

        let mut legality = true;
        //Pawn
        let patt = {
            if my_player_num == 0 {
                self.attack_tables.get_north_pawn_attack_only(loc_index, &position.occupied, &enemies)
            }else{
                self.attack_tables.get_south_pawn_attack_only(loc_index, &position.occupied, &enemies)
            }
        };

        if legality && !(patt & enemy_pawns).is_zero() {
            legality = false;
        };

        //Knight
        let natt = self.attack_tables.get_knight_attack(loc_index, &position.occupied, &enemies);
        if legality && !(natt & enemy_knights).is_zero() {
            legality = false;
        };
        //King
        let katt = self.attack_tables.get_king_attack(loc_index, &position.occupied, &enemies);
        if legality && !(katt & enemy_kings).is_zero() {
            legality = false;
        };

        //Rook & Queen
        let ratt = self.attack_tables.get_rook_attack(loc_index, &position.occupied, &enemies);
        if legality && (!(&ratt & enemy_queens).is_zero() || !(&ratt & enemy_rooks).is_zero()){
            legality = false;
        };
        //Bishop & Queen
        let batt = self.attack_tables.get_bishop_attack(loc_index, &position.occupied, &enemies);
        if legality && (!(&batt & enemy_queens).is_zero() || !(&batt & enemy_bishops).is_zero()) {
            legality = false;
        };
        position.unmake_move();
        legality
    }
    /*
    pub fn get_moves(&self, position:&Position) -> impl Iterator<Item=Move> {
        let my_pieces: &PieceSet = &position.pieces[position.whos_turn as usize];
        let enemies = &position.occupied & !&my_pieces.occupied;

        //create a vector of iterators
        let mut iters:Vec<BitboardMoves> = Vec::new();


        let mut apply_to_each = |mut pieceset:Bitboard, func: fn(&AttackTables, u8, &Bitboard, &Bitboard)-> Bitboard| {
            while !pieceset.is_zero() {
                let index = pieceset.lowest_one().unwrap() as u8;
                let mut raw_attacks = func(&self.attack_tables,index, &position.occupied, &enemies);
                //Do not attack ourselves
                raw_attacks &= !&my_pieces.occupied;
                //Keep only in bounds
                raw_attacks &= &position.bounds;
                iters.push(BitboardMoves{
                    enemies: (&enemies).to_owned(),
                    moves: raw_attacks,
                    source_index: index,
                });
                pieceset.set_bit(index as usize, false);
            }
        };

        apply_to_each((&my_pieces.king).to_owned(), AttackTables::get_king_attack);
        apply_to_each((&my_pieces.queen).to_owned(), AttackTables::get_queen_attack);
        apply_to_each((&my_pieces.rook).to_owned(), AttackTables::get_rook_attack);
        apply_to_each((&my_pieces.bishop).to_owned(), AttackTables::get_bishop_attack);
        apply_to_each((&my_pieces.knight).to_owned(), AttackTables::get_knight_attack);
        if position.whos_turn == 0 {
            apply_to_each((&my_pieces.pawn).to_owned(), AttackTables::get_north_pawn_attack);
        }else{
            apply_to_each((&my_pieces.pawn).to_owned(), AttackTables::get_south_pawn_attack);
        }

        //Flatten our vector of iterators
        iters.into_iter().flatten()
    }
     */
}
