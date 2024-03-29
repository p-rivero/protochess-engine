#[cfg(test)]
mod zobrist_test {
    use std::convert::{TryInto, TryFrom};

    use protochess_engine_rs::{Engine, MoveInfo, MakeMoveResultFlag, GameState};
    
    #[test]
    fn zobrist_pawn_push() {
        let mv = vec!["e2e3"];
        let expected_fen = "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_pawn_double_push() {
        let mv = vec!["e2e4"];
        let expected_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_pawn_capture() {
        let mv = vec!["e2e4", "e7e5", "d2d4", "e5d4"];
        let expected_fen = "rnbqkbnr/pppp1ppp/8/8/3pP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_pawn_en_passant() {
        let mv = vec!["e2e4", "h7h6", "e4e5", "d7d5", "e5d6"];
        let expected_fen = "rnbqkbnr/ppp1ppp1/3P3p/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_castle() {
        let mv = vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "e1h1"];
        let expected_fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b Qkq - 5 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture() {
        let mv = vec!["e2e4", "f7f6", "f1c4", "f6f5", "c4g8"];
        let expected_fen = "rnbqkbBr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture_2() {
        let mv = vec!["e2e4", "f7f6", "f1c4", "f6f5", "c4g8", "h8g8", "e4f5"];
        let expected_fen = "rnbqkbr1/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQK1NR b KQq - 0 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture_3() {
        let mv = vec!["e2e4", "f7f6", "f1c4", "f6f5", "c4g8", "h8g8"];
        let expected_fen = "rnbqkbr1/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQK1NR w KQq - 0 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_capture_rook() {
        // Make sure castling gets disabled for captured rook
        let mv = vec!["b2b3", "g7g6", "c1b2", "g6g5", "b2h8"];
        let expected_fen = "rnbqkbnB/pppppp1p/8/6p1/8/1P6/P1PPPPPP/RN1QKBNR b KQq - 0 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_rook_captures_rook() {
        // Make sure castling gets disabled for both rooks
        let mv = vec!["h2h4", "h7h5", "g2g4", "g7g5", "h4g5", "h5g4", "h1h8"];
        let expected_fen = "rnbqkbnR/pppppp2/8/6P1/6p1/8/PPPPPP2/RNBQKBN1 b Qq - 0 4";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_promotion() {
        let mv = vec!["h2h4", "g7g5", "h4g5", "h7h6", "g5g6", "h6h5", "g6g7", "g8f6", "g7g8=N"];
        let expected_fen = "rnbqkbNr/pppppp2/5n2/7p/8/8/PPPPPPP1/RNBQKBNR b KQkq - 0 5";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_promotion_capture() {
        let mv = vec!["h2h4", "g7g5", "h4g5", "h7h6", "g5g6", "h6h5", "g6g7", "g8f6", "g7f8=B"];
        let expected_fen = "rnbqkB1r/pppppp2/5n2/7p/8/8/PPPPPPP1/RNBQKBNR b KQkq - 0 5";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_promotion_capture_rook() {
        let mv = vec!["h2h4", "g7g5", "h4g5", "h7h6", "g5g6", "h6h5", "g6g7", "g8f6", "g7h8=Q"];
        let expected_fen = "rnbqkb1Q/pppppp2/5n2/7p/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 5";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_transposition() {
        let mv1 = vec!["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "g8f6", "e1h1"];
        let mv2 = vec!["g1f3", "e7e5", "e2e4", "g8f6", "f1c4", "b8c6", "e1h1"];
        let expected_fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQ1RK1 b Qkq - 5 4";
        test_zobrist_sequence(&mv1, expected_fen);
        test_zobrist_sequence(&mv2, expected_fen);
    }
    
    #[test]
    fn zobrist_opposite_player_1() {
        // zobrist_opposite_player_2() results in the same position, but with black to move
        let mv = vec!["e2e3", "d7d6", "f1e2", "c8e6"];
        let expected_fen = "rn1qkbnr/ppp1pppp/3pb3/8/8/4P3/PPPPBPPP/RNBQK1NR w KQkq - 2 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn zobrist_opposite_player_2() {
        // zobrist_opposite_player_1() results in the same position, but with white to move
        // player_to_move_affects_zobrist() tests that the zobrist hash is different for the two
        let mv = vec!["e2e3", "d7d6", "f1d3", "c8e6", "d3e2"];
        let expected_fen = "rn1qkbnr/ppp1pppp/3pb3/8/8/4P3/PPPPBPPP/RNBQK1NR b KQkq - 2 3";
        test_zobrist_sequence(&mv, expected_fen);
    }
    
    #[test]
    fn add_one_piece() {
        let mut engine_start = build_engine_from_fen("rnbqkb1r/pp1ppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb1r/pp1ppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.add_piece('Q', 3, 0, false).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn add_pieces() {
        let mut engine_start = build_engine_from_fen("rnbqkb1r/pp1ppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.add_piece('Q', 3, 0, false).unwrap();
        engine_start.add_piece('n', 6, 7, false).unwrap();
        engine_start.add_piece('p', 2, 6, false).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn add_pieces_rook() {
        let mut engine_start = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NB1KBNR w Kq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb1r/8/8/8/8/8/8/RNB1KBNR w KQkq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.add_piece('R', 0, 0, false).unwrap();
        engine_start.add_piece('r', 7, 7, false).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    #[test]
    fn add_pieces_rook_2() {
        let mut engine_start = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NB1KBNR w Kq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb1r/8/8/8/8/8/8/RNB1KBNR w KQq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.add_piece('R', 0, 0, false).unwrap();
        engine_start.add_piece('r', 7, 7, true).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn add_pieces_king() {
        let mut engine_start = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NB1KBNR w Kq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NBKKBNR w DKq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.add_piece('K', 3, 0, false).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn remove_one_piece() {
        let mut engine_start = build_engine_from_fen("rnbqkb1r/pp1ppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb1r/pp1ppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.remove_piece(3, 0).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn remove_pieces() {
        let mut engine_start = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb1r/pp1ppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.remove_piece(3, 0).unwrap();
        engine_start.remove_piece(6, 7).unwrap();
        engine_start.remove_piece(2, 6).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn remove_pieces_rook() {
        let mut engine_start = build_engine_from_fen("rnbqkb1r/8/8/8/8/8/8/RNB1KBNR w KQkq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NB1KBNR w Kq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.remove_piece(0, 0).unwrap();
        engine_start.remove_piece(7, 7).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn remove_pieces_king() {
        let mut engine_start = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NBKKBNR w Kq - 0 1");
        let engine_end = build_engine_from_fen("rnbqkb2/8/8/8/8/8/8/1NB1KBNR w Kq - 0 1");
        
        assert!(engine_start.get_zobrist() != engine_end.get_zobrist());
        engine_start.remove_piece(3, 0).unwrap();
        assert!(engine_start.get_zobrist() == engine_end.get_zobrist());
    }
    
    #[test]
    fn add_and_remove_vs_move() {
        let mut engine1 = build_engine_from_fen("rnbqkb1r/pp1p1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        let mut engine2 = build_engine_from_fen("rnbqkb1r/pp1p1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        
        engine1.add_piece('P', 3, 2, true).unwrap();
        engine1.remove_piece(3, 1).unwrap();
        
        assert_eq!(engine2.make_move(&"d2d3".try_into().unwrap()).flag, MakeMoveResultFlag::Ok);
        assert!(engine1.get_zobrist() == engine2.get_zobrist());
    }
    
    #[test]
    fn add_and_remove_vs_capture() {
        let mut engine1 = build_engine_from_fen("rnbqkb1r/pp1p1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mut engine2 = engine1.clone();
        assert!(engine1.get_zobrist() == engine2.get_zobrist());
        
        engine1.remove_piece(3, 1).unwrap();
        engine1.remove_piece(4, 4).unwrap();
        engine1.add_piece('p', 3, 3, true).unwrap();
        
        assert_eq!(engine2.make_move(&"d2d4".try_into().unwrap()).flag, MakeMoveResultFlag::Ok);
        assert_eq!(engine2.make_move(&"e5d4".try_into().unwrap()).flag, MakeMoveResultFlag::Ok);
        assert!(engine1.get_zobrist() == engine2.get_zobrist());
    }
    
    #[test]
    fn add_and_remove_vs_move_rook() {
        let mut engine1 = build_engine_from_fen("rnbqkb1r/8/8/4p3/8/8/8/RNBQKBNR b KQkq - 0 1");
        let mut engine2 = build_engine_from_fen("rnbqkb1r/8/8/4p3/8/8/8/RNBQKBNR w KQkq - 0 1");
        
        engine1.add_piece('R', 7, 4, true).unwrap();
        engine1.remove_piece(7, 0).unwrap();
        
        assert_eq!(engine2.make_move(&"h1h5".try_into().unwrap()).flag, MakeMoveResultFlag::Ok);
        assert!(engine1.get_zobrist() == engine2.get_zobrist());
    }
    
    #[test]
    fn add_and_remove_vs_capture_rook() {
        let mut engine1 = build_engine_from_fen("rnbqkb1r/8/8/4p3/8/8/8/RNBQKBNR b KQkq - 0 1");
        let mut engine2 = build_engine_from_fen("rnbqkb1r/8/8/4p3/8/8/8/RNBQKBNR w KQkq - 0 1");
        
        engine1.remove_piece(7, 0).unwrap();
        engine1.remove_piece(7, 7).unwrap();
        engine1.add_piece('R', 7, 7, true).unwrap();
        
        assert_eq!(engine2.make_move(&"h1h8".try_into().unwrap()).flag, MakeMoveResultFlag::Ok);
        assert!(engine1.get_zobrist() == engine2.get_zobrist());
    }
    
    #[test]
    fn castling_rights_affect_zobrist() {
        let mut zobrist = vec![];
        for i in 0..16 {
            let mut castling = String::from("");
            if i & 1 != 0 {
                castling.push('K');
            }
            if i & 2 != 0 {
                castling.push('Q');
            }
            if i & 4 != 0 {
                castling.push('k');
            }
            if i & 8 != 0 {
                castling.push('q');
            }
            if castling.is_empty() {
                castling.push('-');
            }
            let fen = format!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w {castling} - 0 1");
            let engine = build_engine_from_fen(&fen);
            zobrist.push(engine.get_zobrist());
        }
        for i in 0..16 {
            for j in 0..16 {
                if i != j {
                    assert_ne!(zobrist[i], zobrist[j]);
                }
            }
        }
    }
    
    #[test]
    fn ep_square_affects_zobrist() {
        let engine1 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e3 0 1");
        
        assert_ne!(engine1.get_zobrist(), engine2.get_zobrist());
    }
    
    #[test]
    fn player_to_move_affects_zobrist() {
        let engine1 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        
        assert_ne!(engine1.get_zobrist(), engine2.get_zobrist());
    }
    
    #[test]
    fn turn_does_not_affect_zobrist() {
        let engine1 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 2");
        
        assert_eq!(engine1.get_zobrist(), engine2.get_zobrist());
    }
    
    #[test]
    fn halfmove_clock_does_not_affect_zobrist() {
        // Halfmove clock is not implemented
        let engine1 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let engine2 = build_engine_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 1 1");
        
        assert_eq!(engine1.get_zobrist(), engine2.get_zobrist());
    }

    fn test_zobrist_sequence(moves: &[&str], expected_fen: &str) {
        let mut engine1 = Engine::default();
        let mut engine2 = Engine::default();
        let zob_start_1 = engine1.get_zobrist();
        let zob_start_2 = engine2.get_zobrist();
        assert_eq!(zob_start_1, zob_start_2);
        
        for m in moves {
            assert_eq!(engine1.make_move(&MoveInfo::try_from(*m).unwrap()).flag, MakeMoveResultFlag::Ok);
            assert_eq!(engine2.make_move(&MoveInfo::try_from(*m).unwrap()).flag, MakeMoveResultFlag::Ok);
            let zob_1 = engine1.get_zobrist();
            let zob_2 = engine2.get_zobrist();
            assert_eq!(zob_1, zob_2);
        }
        
        let engine3 = build_engine_from_fen(expected_fen);
        assert_eq!(engine1.get_zobrist(), engine3.get_zobrist());
    }
    
    fn build_engine_from_fen(fen: &str) -> Engine {
        let state = GameState::from_debug_fen(fen);
        let mut engine = Engine::default();
        let result = engine.set_state(state).expect("Failed to set state");
        assert_eq!(result.flag, MakeMoveResultFlag::Ok);
        engine
    }
}
