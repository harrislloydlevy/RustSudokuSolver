use crate::constants::*;
use crate::sudoku::*;
use rand::prelude::*;
use rand::seq::SliceRandom;

// Use ChaCha as it can be seeded to be consistent between runs regardless of OS
// and regardless of other versions.

// This file is used to build up new sudoku's for test cases and for fun and profit.
// It uses a combination of seeding base elements of sudokus, randomly adding parts
// and then removing otehr random parts to quickly generate a large variety of
// sudoku's - some of which may even be solvable but all of which should be legally
// constructed.
fn get_9_rands(rng: &mut dyn RngCore) -> [u8; 9] {
    let mut result = ARRAY_OF_9;

    result.shuffle(rng);

    result
}

// Build up random sudoku by filling out some parts of it randomly then trying to
// solve it. Useful for regression testing.
fn build_rand_sud(rng: &mut dyn RngCore) -> Sudoku {
    let mut sud = BLANK_SUDOKU;

    let rands = get_9_rands(rng);

    sud.cells[TOP_LFT].set(rands);
    sud.cells[MID_MID].set(rands);
    sud.cells[BOT_RHT].set(rands);

    return sud;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::*;

    #[test]
    fn test_9_rands() {
        let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(245);

        let result1 = get_9_rands(&mut rng);
        let result2 = get_9_rands(&mut rng);

        println!("Result 1: {:?}", result1);
        println!("Result 2: {:?}", result2);

        assert_ne!(result1, ARRAY_OF_9);
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_rand_sudoku() {
        let mut rng: ChaCha8Rng = ChaCha8Rng::seed_from_u64(0);

        let sud = build_rand_sud(&mut rng);

        assert_eq!(sud.cells[TOP_LFT].boxes[TOP_LFT].value.unwrap(), 4);
        assert_eq!(sud.cells[BOT_RHT].boxes[BOT_RHT].value.unwrap(), 7);
    }
}
