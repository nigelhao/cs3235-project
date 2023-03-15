// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

pub mod miner;


#[cfg(test)]
mod tests {
    use std::thread;
    use std::sync::{Arc, RwLock, Mutex};
    use std::time::Duration;
    
    use crate::miner::{Miner, PuzzleSolution};
    use sha2::{Sha256, Digest};

    // Testing the correctness. To start a multi-threaded and solve the puzzle.
    #[test]
    fn test_miner_wait() {
        let miner_p = Arc::new(Mutex::new(Miner::new()));
        let puzzle = "RANDOM_STRING_ANYTHING".to_owned();
        let cancellation_token = Arc::new(RwLock::new(false));
        let solution = Miner::solve_puzzle(
            miner_p,
            puzzle.clone(),
            16, 3, 6, 43,  
            cancellation_token
        ).unwrap();

        let PuzzleSolution { puzzle, nonce, hash } = solution;
        let mut hasher = Sha256::new();
        hasher.update(&nonce);
        hasher.update(&puzzle);
        let result = hasher.finalize();
        let generated_hash: String = format!("{:x}", result);
        assert!(generated_hash.starts_with("000") && generated_hash == hash);
        println!("HASH: {}\nNONCE: {}\nPUZZLE: {}", hash, nonce, puzzle);
    }

   
   // Testing cancellation. To start a multi-threaded miner and solve a very hard puzzle and cancel it after 4 secs.
    #[test]
    fn test_miner_cancellation() {
        let miner_p = Arc::new(Mutex::new(Miner::new()));
        let puzzle = "RANDOM_STRING_ANYTHING".to_owned();
        let cancellation_token = Arc::new(RwLock::new(false));
        let cancellation_token_cloned = cancellation_token.clone();

        // create a thread to cancel solving after 4 secs
        let _cancel_timer = thread::spawn(move || {
            for i in 0..4 {
                println!("Wait sec {}...", i);
                thread::sleep(Duration::from_millis(1000));
            }
            println!("Cancel solving if not finished.");
            let mut writable = cancellation_token_cloned.write().unwrap();
            *writable = true;
        });

        // start mining with the cancellation token
        let solution = Miner::solve_puzzle(
            miner_p,
            puzzle.clone(),
            16, 8, 2, 43,  
            cancellation_token
        );

        // The execution should reach here after 4s even if solution is not found!
        // check if solved or cancelled
        match solution {
            Some(PuzzleSolution { puzzle, nonce, hash }) => {println!("Solution Found! HASH: {}\nNONCE: {}\nPUZZLE: {}", hash, nonce, puzzle);},
            None => {println!("Miner returns None. Expected.");}
        };
        _cancel_timer.join().unwrap();
    }


    /// Your own additional test that tests your implementation more throughly (e.g. any performance issue in multi-threading)
    #[test]
    fn test_miner_additional() {
        // Please fill in the blank
        
    }

}

