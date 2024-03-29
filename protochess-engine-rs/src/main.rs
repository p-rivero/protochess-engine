//#[macro_use] extern crate scan_rules;

use std::io::Write;

use protochess_engine_rs::{Engine, MoveInfo, MakeMoveResultFlag, MakeMoveResultWinner, GameState};
use protochess_engine_rs::types::GameMode;
use protochess_engine_rs::utils::debug::split_debug_fen;

pub fn main() {
    
    // Some interesting FENs:
    // "R3b3/4k3/2n5/p4p1p/4p3/2B5/1PP2PPP/5K2 w - - 10 36"
    // "rnbqkbnr/nnnnnnnn/rrrrrrrr/8/8/8/QQQQQQQQ/RNBQKBNR w KQkq - 0 1"
    // "rnbqkbnr/pp4pp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    // "r1b3nr/ppqk1Bbp/2pp4/4P1B1/3n4/3P4/PPP2QPP/R4RK1 w - - 1 0"
    // "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    // "rnbqkbnr/pppppppp/8/8/8/8/8/RNBQKBNR w KQkq - 0 1"
    
    
    // Usage: cargo run -- <depth> <fen> <num_ply>
    // By default, <depth> is 12, <fen> is the starting position, and <num_ply> is 500
    // Example: cargo run -- 4 "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    
    let mut pgn_file = std::fs::File::create("pgn.txt").expect("create failed");

    
    let args: Vec<String> = std::env::args().collect();
    let mut fixed_depth = true;
    let mut depth = 12;
    let mut max_ply = 500;
    if args.len() > 3 {
        max_ply = args[3].parse::<u32>().unwrap();
    }
    
    
    let mut engine = Engine::default();
    
    if args.len() > 2 && args[2] != "default" {
        let state = GameState::from_debug_fen(&args[2]);
        print_pgn_header(&args[2], &mut pgn_file);
        let _result = engine.set_state(state).unwrap_or_else(|e| {
            println!("Incorrect FEN: {e}");
            std::process::exit(1);
        });
    }
    if args.len() > 1 {
        if args[1].contains('t') {
            fixed_depth = false;
            depth = args[1].replace('t', "").parse::<u8>().unwrap();
        } else {
            depth = args[1].parse::<u8>().unwrap();
        }
    }
    
    println!("Start Position:\n{engine}");
    println!("\n----------------------------------------\n");
    
    let start = instant::Instant::now();
    for ply in 0..max_ply {
        let mv = {
            if fixed_depth {
                engine.get_best_move(depth).unwrap().0
            } else {
                engine.get_best_move_timeout(depth as u64).unwrap().0
            }
        };
        println!("\n========================================\n");
        println!("(Time since start: {:?})", start.elapsed());
        println!("PLY: {ply} Engine plays:\n");
        let result = engine.make_move(&mv);
        let move_str = result.move_notation.unwrap_or_else(|| "!!!".to_string());
        print_pgn(&mut pgn_file, ply, &fix_notation(move_str, mv));
        println!("{engine}\n");
        match result.flag {
            MakeMoveResultFlag::Ok => {
                println!("----------------------------------------\n");
            },
            MakeMoveResultFlag::IllegalMove => {
                panic!("An illegal move was made");
            },
            MakeMoveResultFlag::Checkmate => {
                println!("CHECKMATE! {:?} wins!", result.winner);
                break;
            },
            MakeMoveResultFlag::LeaderCaptured => {
                println!("KING HAS BEEN CAPTURED! {:?} wins!", result.winner);
                break;
            },
            MakeMoveResultFlag::AllPiecesCaptured => {
                println!("ALL PIECES HAVE BEEN CAPTURED! {:?} wins!", result.winner);
                break;
            },
            MakeMoveResultFlag::PieceInWinSquare => {
                println!("KING IN WINNING SQUARE! {:?} wins!", result.winner);
                break;
            },
            MakeMoveResultFlag::CheckLimit => {
                println!("CHECK LIMIT REACHED! {:?} wins!", result.winner);
                break;
            },
            MakeMoveResultFlag::Stalemate => {
                if result.winner == MakeMoveResultWinner::None {
                    println!("DRAW BY STALEMATE!");
                } else {
                    println!("STALEMATE! {:?} wins!", result.winner);
                }
                break;
            },
            MakeMoveResultFlag::Repetition => {
                println!("DRAW BY REPETITION!");
                break;
            },
        }
    }
}


fn print_pgn_header(fen: &str, pgn_file: &mut std::fs::File) {
    let (fen, variant) = split_debug_fen(fen);
    if variant != GameMode::Standard {
        pgn_file.write_all(format!("[Variant \"{variant}\"]\n").as_bytes()).unwrap();
    }
    pgn_file.write_all(format!("[FEN \"{fen}\"]\n\n").as_bytes()).unwrap();
}

fn print_pgn(pgn_file: &mut std::fs::File, ply: u32, move_str: &str) {
    if (ply % 2) == 0 {
        let round = format!("{}. ", ply/2 + 1);
        pgn_file.write_all(round.as_bytes()).expect("write failed");
    }
    pgn_file.write_all(move_str.as_bytes()).expect("write failed");
    pgn_file.write_all(b" ").expect("write failed");
}


// Algebraic notation specifies that a pawn capture includes the file of the capturing pawn,
// even if the capture is unambiguous without it. This is not generalizable at all, so
// the engine does not include the file in the notation.
// If we want Lichess to accept our PGNs, we need to add the file back in.
fn fix_notation(move_str: String, mv: MoveInfo) -> String {
    if move_str.starts_with('x') {
        format!("{}{}", (b'a' + mv.from.0) as char, move_str)
    } else {
        move_str
    }
}
