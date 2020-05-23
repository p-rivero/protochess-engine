use arrayvec::ArrayVec;
use crate::types::*;
use crate::constants::{fen, DEFAULT_WIDTH, DEFAULT_HEIGHT};
use crate::position::piece_set::PieceSet;
use crate::types::bitboard::{Bitboard, to_index, from_index, to_string};
use std::sync::Arc;

use position_properties::PositionProperties;
use crate::types::chess_move::{Move, MoveType};

mod position_properties;
mod castle_rights;
pub mod piece_set;
use crate::position::piece_set::movement_pattern::MovementPattern;
/// Represents a single position in chess
pub struct Position {
    pub dimensions: Dimensions,
    pub bounds: Bitboard, //Bitboard representing the boundaries
    pub num_players: u8,
    pub whos_turn: u8,
    pub pieces:ArrayVec<[PieceSet;4]>, //pieces[0] = white's pieces, pieces[1] black etc
    pub occupied: Bitboard,
    //Properties relating only to the current position
    // Typically hard-to-recover properties, like castling
    //Similar to state in stockfish
    pub properties: Arc<PositionProperties>,
}

impl Position {
    pub fn default() -> Position{
        Position::from_fen(String::from(fen::STARTING_POS))
    }

    /// Registers a new piece type for this position
    pub fn register_piecetype(&mut self, player_num: usize, char_rep: char, mp: MovementPattern) {
        self.pieces[player_num].custom.push((char_rep, Bitboard::zero(),mp));
    }

    /// Modifies the position to make the move
    pub fn make_move(&mut self, move_: Move) {
        let my_player_num = self.whos_turn;
        self.whos_turn = (self.whos_turn + 1) % self.num_players;

        let mut new_props:PositionProperties = (*self.properties).clone();
        //In the special case of the null move, don't do anything except update whos_turn
        //And update props
        if move_.get_move_type() == MoveType::Null {
            //Update props
            new_props.move_played = Some(move_);
            new_props.prev_properties = Some(Arc::clone(&self.properties));
            self.properties = Arc::new(new_props);
            return;
        }

        //Special moves
        match move_.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt_index = move_.get_target();
                new_props.captured_piece = self.piece_at(capt_index as usize);
                self.remove_piece(capt_index);
            },
            MoveType::KingsideCastle => {
                let rook_from = move_.get_target();
                let (x, y) = from_index(move_.get_to() as usize);
                let rook_to = to_index(x - 1, y) as u8;
                self.move_piece(rook_from,rook_to);
            },
            MoveType::QueensideCastle => {
                let rook_from = move_.get_target();
                let (x, y) = from_index(move_.get_to() as usize);
                let rook_to = to_index(x + 1, y) as u8;
                self.move_piece(rook_from,rook_to);
            }
            _ => {}
        }

        let from= move_.get_from();
        let to = move_.get_to();
        let from_piece_type = self.piece_at(from as usize).unwrap().1;

        //Move piece to location
        self.move_piece(from, to);
        //Promotion
        match move_.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                new_props.promote_from = Some(from_piece_type.to_owned());
                self.remove_piece(to);
                self.add_piece(my_player_num, PieceType::from_char(move_.get_promotion_char().unwrap()), to);
            },
            _ => {}
        };

        //Pawn en-passant
        //Check for a pawn double push to set ep square
        let (x1, y1) = from_index(from as usize);
        let (x2, y2) = from_index(to as usize);
        if from_piece_type == PieceType::Pawn
            && (y2 as i8 - y1 as i8).abs() == 2
            && x1 == x2 {
            new_props.ep_square = Some(
                if y2 > y1 {
                    to_index(x1, y2 - 1) as u8
                } else {
                    to_index(x1, y2 + 1) as u8
                }
            );
        } else {
            new_props.ep_square = None;
        }

        //Castling
        //Disable rights if applicable
        if new_props.castling_rights.can_player_castle(my_player_num) {
            if from_piece_type == PieceType::King {
                new_props.castling_rights.disable_kingside_castle(my_player_num);
                new_props.castling_rights.disable_queenside_castle(my_player_num);
            }else if from_piece_type == PieceType::Rook {
                //King side
                if x1 >= self.dimensions.width/2 {
                    new_props.castling_rights.disable_kingside_castle(my_player_num);
                }else{
                    new_props.castling_rights.disable_queenside_castle(my_player_num);
                }
            }
        }

        //Update props
        new_props.move_played = Some(move_);
        new_props.prev_properties = Some(Arc::clone(&self.properties));
        self.properties = Arc::new(new_props);
        //Update occupied bbs for future calculations
        self.update_occupied();
    }

    /// Undo the most recent move
    pub fn unmake_move(&mut self) {

        if self.whos_turn == 0 {
            self.whos_turn = self.num_players -1;
        }else{
            self.whos_turn = (self.whos_turn - 1) % self.num_players;
        }

        let my_player_num = self.whos_turn;
        let move_ = self.properties.move_played.unwrap();
        //Undo null moves
        if move_.get_move_type() == MoveType::Null {
            //Update props
            //Consume prev props; never to return again
            self.properties = self.properties.get_prev().unwrap();
            return;
        }
        let from = move_.get_from();
        let to= move_.get_to();

        //Undo move piece to location
        //Remove piece here
        self.move_piece(to, from);
        //Undo Promotion
        match move_.get_move_type() {
            MoveType::PromotionCapture | MoveType::Promotion => {
                self.remove_piece(from);
                self.add_piece(my_player_num, self.properties.promote_from.as_ref().unwrap().to_owned(), from);
            },
            _ => {}
        };

        //Undo special moves
        //Special moves
        match move_.get_move_type() {
            MoveType::Capture | MoveType::PromotionCapture => {
                let capt = move_.get_target();
                let (owner, pt) = self.properties.captured_piece.as_ref().unwrap();
                self.add_piece(*owner, pt.to_owned(), capt);
            },
            MoveType::KingsideCastle => {
                let rook_from = move_.get_target();
                let (x, y) = from_index(move_.get_to() as usize);
                let rook_to = to_index(x - 1, y) as u8;
                self.move_piece(rook_to,rook_from);
            },
            MoveType::QueensideCastle => {
                let rook_from = move_.get_target();
                let (x, y) = from_index(move_.get_to() as usize);
                let rook_to = to_index(x + 1, y) as u8;
                self.move_piece(rook_to,rook_from);
            }
            _ => {}
        }

        //Update props
        //Consume prev props; never to return again
        self.properties = self.properties.get_prev().unwrap();

        //Update occupied bbs for future calculations
        self.update_occupied();
    }

    pub fn to_string(&self) -> String {
        let mut return_str= String::new();
        for y in (0..self.dimensions.height).rev() {
            for x in 0..self.dimensions.width {

                if let Some((i, pt)) = self.piece_at(bitboard::to_index(x,y)){
                    match pt {
                        PieceType::King => if i == 0 {return_str.push('K')} else {return_str.push('k')},
                        PieceType::Queen => if i == 0 {return_str.push('Q')} else {return_str.push('q')},
                        PieceType::Rook => if i == 0 {return_str.push('R')} else {return_str.push('r')},
                        PieceType::Bishop => if i == 0 {return_str.push('B')} else {return_str.push('b')},
                        PieceType::Knight => if i == 0 {return_str.push('N')} else {return_str.push('n')},
                        PieceType::Pawn => if i == 0 {return_str.push('P')} else {return_str.push('p')},
                        PieceType::Custom(c) => return_str.push(c),
                    }
                }else{
                    return_str.push('.');
                }
                return_str.push(' ');
            }
            return_str.push('\n');
        }
        return_str
    }

    pub fn from_fen(fen: String) -> Position{
        let dims = Dimensions{width:DEFAULT_WIDTH,height:DEFAULT_HEIGHT};

        let mut wb_pieces = ArrayVec::<[_;4]>::new();
        let mut w_pieces = PieceSet::new();
        let mut b_pieces = PieceSet::new();

        let mut x:u8 =0;
        let mut y :u8 = 7;
        let mut field = 0;

        let mut whos_turn = 0;
        let mut ep_sq = 0;
        let mut can_w_castleK = false;
        let mut can_b_castleK = false;
        let mut can_w_castleQ = false;
        let mut can_b_castleQ = false;
        for c in fen.chars(){
            if c == ' ' {
                field += 1;
            }
            match field{
                //position
                0 => {
                    if c == '/' {
                        x = 0;
                        y -= 1;
                        continue;
                    }else if c.is_numeric() {
                        x += c.to_digit(10).expect("Not a digit!") as u8;
                        continue;
                    }

                    let index = bitboard::to_index(x, y);
                    let bitboard: &mut Bitboard = match c.to_ascii_lowercase() {
                        'k' => {
                            if c.is_uppercase() { &mut w_pieces.king } else { &mut b_pieces.king }
                        },
                        'q' => {
                            if c.is_uppercase() { &mut w_pieces.queen } else { &mut b_pieces.queen }
                        },
                        'r' => {
                            if c.is_uppercase() { &mut w_pieces.rook } else { &mut b_pieces.rook }
                        },
                        'b' => {
                            if c.is_uppercase() { &mut w_pieces.bishop } else { &mut b_pieces.bishop }
                        },
                        'n' => {
                            if c.is_uppercase() { &mut w_pieces.knight } else { &mut b_pieces.knight }
                        },
                        'p' => {
                            if c.is_uppercase() { &mut w_pieces.pawn } else { &mut b_pieces.pawn }
                        },
                        _ => continue,
                    };

                    bitboard.set_bit(index, true);
                    if c.is_uppercase() {w_pieces.occupied.set_bit(index,true)} else {b_pieces.occupied.set_bit(index, true)};
                    x += 1;
                }
                //next to move
                1 => {
                    if c == 'w' {
                        whos_turn = 0;
                    }else{
                        whos_turn = 1;
                    }
                }
                //Castling rights
                2 => {
                    match c {
                        'K' => {can_w_castleK = true;}
                        'Q' => {can_w_castleQ = true;}
                        'k' => {can_b_castleK = true;}
                        'q' => {can_b_castleQ = true;}
                        _ => {}
                    }
                }
                //EP square
                3 => {
                    //TODO


                }
                _ => continue,
            }
        }

        let mut occupied = Bitboard::zero();
        occupied |= &w_pieces.occupied;
        occupied |= &b_pieces.occupied;

        wb_pieces.push(w_pieces);
        wb_pieces.push(b_pieces);

        let mut bounds = Bitboard::zero();
        for x in 0..8{
            for y in 0..8{
                bounds.set_bit(to_index(x,y),true);
            }
        }


        let mut properties = PositionProperties::default();
        if !can_w_castleK {
            properties.castling_rights.disable_kingside_castle(0);
        }

        if !can_b_castleK {
            properties.castling_rights.disable_kingside_castle(1);
        }

        if !can_w_castleQ {
            properties.castling_rights.disable_queenside_castle(0);
        }

        if !can_b_castleQ {
            properties.castling_rights.disable_queenside_castle(1);
        }

        /*
        println!("to_move: {}\nwhite: \n    K: {} Q: {} \nblack: \n    K: {} Q: {}",
                 whos_turn,
                 properties.castling_rights.can_player_castle_kingside(0),
                 properties.castling_rights.can_player_castle_queenside(0),
                 properties.castling_rights.can_player_castle_kingside(1),
                 properties.castling_rights.can_player_castle_queenside(1));
         */

        let mut pos = Position{
            whos_turn,
            num_players: 2,
            dimensions: dims,
            pieces: wb_pieces,
            occupied,
            bounds,
            properties: Arc::new(properties)
        };

        pos
    }

    /// Returns tuple (player_num, PieceType)
    pub fn piece_at(&self,index:usize) -> Option<(u8, PieceType)> {
        for (i, ps) in self.pieces.iter().enumerate() {
            if let Some(c) = ps.piecetype_at(index) {
                return Some((i as u8, c));
            }
        }
        None
    }

    /// Returns bitoard of piece at index
    pub fn piece_bb_at(&mut self,index:usize) -> Option<&mut Bitboard> {
        for (i, ps) in self.pieces.iter_mut().enumerate() {
            if let Some(b) = ps.piece_bb_at(index) {
                return Some(b);
            }
        }
        None
    }

    /// Returns if the point is in bounds
    pub fn xy_in_bounds(&self, x:u8, y:u8) -> bool {
        self.bounds.bit(to_index(x, y)).unwrap()
    }


    pub fn move_piece(&mut self, from:u8, to:u8){
        if let Some(source_bb) = self.piece_bb_at(from as usize){
            source_bb.set_bit(from as usize, false);
            source_bb.set_bit(to as usize, true);
        }else{
            println!("nothing to move??");
            println!("from {} {}", from_index(from as usize).0, from_index(from as usize).1);
            println!("to {} {}", from_index(to as usize).0, from_index(to as usize).1);
            println!("==");
        }
    }

    /// Removes a piece from the position, assuming the piece is there
    pub fn remove_piece(&mut self, index:u8) {
        let capd_bb:&mut Bitboard = self.piece_bb_at(index as usize).unwrap();
        capd_bb.set_bit(index as usize, false);
    }

    /// Adds a piece to the position, assuming the piecetype already exists
    /// Panics if the piecetype isn't registered already
    pub fn add_piece(&mut self, owner:u8, pt: PieceType, index:u8){
        match pt {
            PieceType::King => {self.pieces[owner as usize].king.set_bit(index as usize, true);},
            PieceType::Queen => {self.pieces[owner as usize].queen.set_bit(index as usize, true);},
            PieceType::Rook => {self.pieces[owner as usize].rook.set_bit(index as usize, true);},
            PieceType::Bishop => {self.pieces[owner as usize].bishop.set_bit(index as usize, true);},
            PieceType::Knight => {self.pieces[owner as usize].knight.set_bit(index as usize, true);},
            PieceType::Pawn => {self.pieces[owner as usize].pawn.set_bit(index as usize, true);},
            PieceType::Custom(ptc) => {
                for (c, bb, mP) in self.pieces[owner as usize].custom.iter_mut() {
                    if ptc == *c {
                        bb.set_bit(index as usize,true);
                        break;
                    }
                }
            },
        }
    }

    fn update_occupied(&mut self){
        self.occupied = Bitboard::zero();
        for (i, ps) in self.pieces.iter_mut().enumerate() {
            ps.update_occupied();
            self.occupied |= &ps.occupied;
        }
    }
}

