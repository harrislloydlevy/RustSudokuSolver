use crate::constants::*;
use rand::seq::SliceRandom;
use rand::prelude::*;

// Use ChaCha as it can be seeded to be consistent between runs regardless of OS
// and regardless of other versions.
use rand_chacha::*;

// This file is used to build up new sudoku's for test cases and for fun and profit.
// It uses a combination of seeding base elements of sudokus, randomly adding parts
// and then removing otehr random parts to quickly generate a large variety of
// sudoku's - some of which may even be solvable but all of which should be legally
// constructed.

fn get_9_rands(rng: &mut dyn RngCore) -> [i32; 9] {
    let mut result = ARRAY_OF_9;

    result.shuffle(rng);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_9_rands() {
        let mut rng:ChaCha8Rng = ChaCha8Rng::seed_from_u64(245);

        let result1 = get_9_rands(&mut rng);
        let result2 = get_9_rands(&mut rng);

        println!("Result 1: {:?}", result1);
        println!("Result 2: {:?}", result2);

        assert_ne!(result1, ARRAY_OF_9);
        assert_ne!(result1, result2);
    }
}
