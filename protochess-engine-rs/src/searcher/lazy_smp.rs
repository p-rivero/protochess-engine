use std::sync::atomic::{Ordering::Relaxed};
use std::thread;
use instant::{Instant, Duration};

use crate::types::{Move, Depth, GameResult, SearcherError, Centipawns};
use crate::position::Position;
use crate::move_generator::MoveGenerator;
use crate::evaluator::Evaluator;

use super::{Searcher, init_globals, transposition_table, GLOBAL_DEPTH, SEARCH_ID};

// This file contains the multi-threaded search using Lazy SMP, which uses the alphabeta() function from alphabeta.rs


static NUM_THREADS: u32 = 4;

impl Searcher {
    pub fn get_best_move(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, depth: Depth) -> Result<(Move, Depth), GameResult> {
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::get_best_move_impl(position, eval, movegen, depth, 1_000_000)
    }

    pub fn get_best_move_timeout(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, time_sec: u64) -> Result<(Move, Depth), GameResult> {
        Searcher::get_best_move_impl(position, eval, movegen, Depth::MAX, time_sec)
    }
    
    fn get_best_move_impl(position: &Position, eval: &Evaluator, movegen: &MoveGenerator, max_depth: Depth, time_sec: u64) -> Result<(Move, Depth), GameResult> {
        
        // Initialize the global structures
        init_globals();
        
        // Init threads, store handles
        let mut handles = Vec::with_capacity(NUM_THREADS as usize);
        for thread_id in 0..NUM_THREADS {
            // For each thread, create a local copy of the heuristics
            let mut pos_copy = position.clone();
            let mut eval_copy = eval.clone();
            let movegen_copy = (*movegen).clone();
            let h = thread::spawn(move || {
                let mut searcher = Searcher::new();
                searcher.search_thread(thread_id, &mut pos_copy, &mut eval_copy, &movegen_copy, max_depth, time_sec)
            });
            handles.push(h);
        }
        
        // Wait for threads to finish
        let mut best_move = Move::null();
        let mut best_score = -Centipawns::MAX; // Use -MAX instead of MIN to avoid overflow when negating
        let mut best_depth = 0;
        for h in handles {
            // If any thread returns a GameResult, drop the other threads and return the result
            let (mv, score, depth) = h.join().unwrap()?;
            // If any thread reaches the target depth, drop the other threads and return the move
            if depth == max_depth {
                return Ok((mv, depth));
            }
            if depth > best_depth || (depth == best_depth && score > best_score) {
                best_move = mv;
                best_score = score;
                best_depth = depth;
            }
        }
        Ok((best_move, best_depth))
    }
    
    // Run for some time, then return the best move, its score, and the depth
    fn search_thread(&mut self, thread_id: u32, position: &mut Position, eval: &mut Evaluator, movegen: &MoveGenerator, max_depth: Depth, time_sec: u64) -> Result<(Move, Centipawns, Depth), GameResult> {
        let end_time = Instant::now() + Duration::from_secs(time_sec);
        
        let mut best_move: Move = Move::null();
        let mut best_score: Centipawns = 0;
        let mut best_depth: Depth = 0;
        
        // At the start, each thread should search a different depth (between 1 and max_depth, inclusive)
        let mut local_depth = (thread_id as Depth % max_depth) + 1;
        
        loop {
            self.nodes_searched = 0;
            self.current_searching_depth = local_depth;
            match crate::searcher::alphabeta(self, position, eval, movegen, local_depth, -Centipawns::MAX, Centipawns::MAX, true, &end_time) {
                Ok(score) => {
                    best_score = score;
                    best_move = transposition_table().retrieve(position.get_zobrist()).unwrap().mv;
                    best_depth = local_depth;
                },
                Err(SearcherError::Timeout) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
                Err(SearcherError::Checkmate) => {
                    assert!(best_depth == 0);
                    return Err(GameResult::Checkmate);
                },
                Err(SearcherError::Stalemate) => {
                    assert!(best_depth == 0);
                    return Err(GameResult::Stalemate);
                }
            }
            //Print PV info
            println!("Thread {} score: {}, depth: {}, nodes: {}", thread_id, best_score, best_depth, self.nodes_searched);
            
            // Set the global depth to max(local_depth, global_depth)
            // GLOBAL_DEPTH contains the maximum depth searched by any thread
            let old_global_depth = unsafe { GLOBAL_DEPTH.fetch_max(local_depth, Relaxed) };
            
            // If time is up or any thread has searched to max_depth, return
            if Instant::now() >= end_time || local_depth == max_depth || old_global_depth == max_depth {
                // Signal to other threads that they can stop
                unsafe { GLOBAL_DEPTH.store(max_depth, Relaxed); }
                break;
            }
                        
            if NUM_THREADS == 1 {
                // Iterative deepening
                local_depth += 1;
            } else {
                // Update local_depth: set to GLOBAL_DEPTH + increment
                let search_id = unsafe { SEARCH_ID.fetch_add(1, Relaxed) };
                // 1/2 threads search 1 ply deeper, 1/4 threads search 2 ply deeper, etc.
                let increment = 1 + search_id.trailing_zeros() as Depth;
                local_depth = unsafe { GLOBAL_DEPTH.load(Relaxed) } + increment;
            }
            // Limit local_depth to max_depth
            local_depth = std::cmp::min(local_depth, max_depth);
        }

        Ok((best_move.to_owned(), best_score, best_depth))
    }
}