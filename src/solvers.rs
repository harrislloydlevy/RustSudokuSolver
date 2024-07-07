use crate::constants::*;
use crate::sk_box::Box;
use crate::sudoku::Sudoku;

/*
 * Solving technique names taken from sudokuoftheday.com. Logic and code is mine
 * but I've used the names from there so if i explain it badly someone else can
 * google a description with pictures.
 */

// useful enum sometimes for switching on solving
#[derive(PartialEq, Debug, Copy, Clone)]
enum Direction {
    HOR,
    VER,
}

// Useful enum for how many times a value has been found
#[derive(PartialEq, Debug, Copy, Clone)]
enum Found {
    NONE,
    ONCE,
    MANY,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct PossValWhere {
    index: Option<u8>,
    found: Found,
}

pub const BLANK_PVW: PossValWhere = PossValWhere {
    index: None,
    found: Found::NONE,
};

/**
 * Simply do a quick "normalise" over the sudoku to remove all of the possible
 * tags against all the boxes for solved values in each cell.
 */
pub fn normalise(sudoku: &mut Sudoku) {
    for i in 0..9 {
        single_position_array(sudoku.cells[i].get_mut());
    }
}

/**
 *
 * single_position
 *
 * Perform one round of the most naive solving of the sudoku possible.
 * Only applies the three basic set solving techniques to each cell, row
 * and column of the sudoku.
 * 1. If there is a solved value in the set of 9, remove that as an option
 *    from every other box.
 * 2. If there is only one box that could hold a particular value then
 *    set that box to the value. (Single position candidate)
 */
pub fn single_position(sudoku: &mut Sudoku) {
    for i in 0..9 {
        single_position_array(sudoku.cells[i].get_mut());
    }

    for i in 0..9 {
        // How does this not need a mut???
        single_position_array(sudoku.get_row_mut(i));
    }

    for i in 0..9 {
        // How does this not need a mut???
        single_position_array(sudoku.get_col_mut(i));
    }
}

/**
 *
 * naked_set
 *
 * 3. If there is a set of N values that are the only possible in the same N
 *    squares, then remove them as possibles from all other squares (Naked set)
 */
pub fn naked_set(sudoku: &mut Sudoku) {
    for i in 0..9 {
        naked_set_array(sudoku.cells[i].get_mut());
    }

    for i in 0..9 {
        // How does this not need a mut???
        naked_set_array(sudoku.get_row_mut(i));
    }

    for i in 0..9 {
        // How does this not need a mut???
        naked_set_array(sudoku.get_col_mut(i));
    }
}

/**
 * single_position_array
 *
 * This is the dumbest solving algorithim there is.
 *
 * It looks over a list of 9
 * boxes and for any boxes that are solved, it removes the solved boxes from
 * thee list of possible for the other boxes. If one of the boxes now only
 * has a single possible value then it's set as solved to that value.
 *
 * When run over every cell, row, and column it implmeents the single Position
 * and single_candidate logic.
 */
pub fn single_position_array(mut boxes: Vec<&mut Box>) {
    // pos_vals is the bit mask of still possible values in this set of interlinked
    // boxes.
    //
    // Each solved valuewill zero out it's own value in the mask so as to mark it as
    // not possible in the unsovled values in the cell.
    //
    // We iterate over the boxes remove confirmed values from all of them
    // returning a map of only the possible values in this set of boxes.
    let mut poss_vals: u16 = 0b1111111110;

    // For finding out which values could only occur in one location we setup an
    // array of which boxes could be each value. The array is indexed by value (1-9)
    // and each element in it points to a struct taht says the index within the array that holds the
    // single element that could be that value. Plus an additional element that
    // says whether the value has been found yet, found once, or more than once.
    // Obvioulsy we only care if it's found once! but need a ternary value to differentiate
    // not found.
    //
    // As always I am lazy, so all arrays indexed by possible value have 10 elemented with the
    // 0th elemnent unused.
    let mut last_poss_vals: [PossValWhere; 10] = [BLANK_PVW; 10];

    for x in boxes.iter() {
        // If we have an actual value we blank out that possible value from the map
        // otherwise ignore the uncionfirmed values.
        match x.value {
            // mask it off against the inverse of the found value.
            Some(confirmed_value) => poss_vals &= !(ON << confirmed_value),
            None => (),
        };
    }

    // Now in poss_vals we have an bitmap that represents all the values that nothing
    // else can be. So we apply that to each of the values in the 9 array
    // set that are still looking for a value.
    for cur_idx in 0..9 {
        let unsolved_box = &mut boxes[cur_idx];
        // If we have an unconfirmed values remove the possibilities foumd, otherwise
        // for solved boxes we just skip over.
        match unsolved_box.value {
            Some(_unused) => {}
            None => {
                unsolved_box.remove_possible_bits(poss_vals);

                for poss_val in unsolved_box.get_possibles() {
                    let lpv = &mut last_poss_vals[poss_val as usize];
                    match (*lpv).found {
                        Found::NONE => {
                            lpv.found = Found::ONCE;
                            lpv.index = Some(cur_idx as u8);
                        }
                        Found::ONCE => {
                            lpv.found = Found::MANY;
                            lpv.index = None;
                        }
                        Found::MANY => {}
                    }
                }
            }
        }
    }

    // We are now done interating over the boxes, and can check the LPV
    // array for any elements that have only been found once.
    for cur_val in 1..=9 {
        let lpv = last_poss_vals[cur_val];

        if lpv.found == Found::ONCE {
            // We have a value that had been found once! The LPV will tell us the index in the
            // boxes.
            boxes[(lpv.index.unwrap()) as usize].set_val(cur_val as u8);
        }
    }
}

/**
 * naked_set_array
 *
 * Looks for combinations of boxes where there are only X boxes that can
 * only be a set combination of X values, then remove that possible combination
 * from all other boxes.
 *
 * e.g. if there is a set of 9 boxes, but 2 of them can *only* be 6 or 9, then
 * logically none of the other boxes in the set can be 6 or 9, so remove them as
 * options. This also applies for groups of 3 or 4.
 *
 * Groups of 5 or 6 are possible, but so rare and computationally expensive we don't bother.
 * https://www.sudokuoftheday.com/techniques/naked-pairs-triples/
 */
fn naked_set_array(mut boxes: Vec<&mut Box>) {
    /*
     * The logic we will follow for this function is as follows:
     *  - Iterate over every number of factorials we'll look for 2, 3, and 4
     *    - Then iterate over every possilbe combination of 1-9 of that number of factorials
     *       - Then check how many boxes match that combination exactly
     *       - And If that matches the number of factoritals then:
     *         - remove that combination from all other boxes.
     *
     * For convenience of checkign we'll primarially use bit patterns, and arrays of bit patterns.
     */
    assert!(boxes.len() == 9);

    for factors in vec![2, 3, 4, 5] {
        let bit_patterns = combo(&[1, 2, 3, 4, 5, 6, 7, 8, 9], factors);

        for pattern in bit_patterns.iter() {
            // Get all the boxes in our input that match that bit pattern exactly.
            // TODO: This is actually not complete for triple/quads! For example a set of
            //       three boxes could be {1,4}/{4,7},{7,1} and so this set of three would
            //       only ever be 1 4 or 7, so qualiyf as a naked triple, but this wouldn't
            //       be caught by this test.
            let matches = boxes
                .iter()
                .filter(|x| x.get_possibles_bits() == *pattern)
                .count() as u16;

            // If we every find more matches than there are facotrs something has gone *very*
            // wrong upstream. It would mean N boxes are vyiung for N+1 values which isn't right.
            assert!(matches <= factors);

            // If there are exactly as many as  we are looking for (hardcoded to 4 right now)
            //then remove this bit pattern as a possibility from all other boxes in the collection.
            if matches == factors {
                // Find the boxes that didn't match the pattern exactly.
                boxes
                    .iter_mut()
                    .filter(|x| x.get_possibles_bits() != *pattern)
                    .for_each(|x| x.remove_impossible_bits(*pattern));
            }
        }
    }
}

/**
 * More generic combinations function used in combinations_maps that calls itself recursively
 * to get all the combinartion bitmaps.
 *
 * Code not copied/pasted from rosetta-code, but algorithim and ideas from there.
 *
 * pool: The list of numbers left to choose from
 * need: The number of combinations needed
 * returns: A list of u16s, each one is a bitmap the number of needed values from the pool of possible values
 *          e.g. 000011 would mean a combination of 1 and 2, 1011, a combiantions of 4, 2, and 1, 100001 = 6 and 1.
 */
fn combo(pool: &[u16], k: u16) -> Vec<u16> {
    let mut result = Vec::new();

    if k > (pool.len() as u16) {
        return result;
    }

    for i in 0..pool.len() {
        if k == 1 {
            result.push(ON << pool[i]);
        } else {
            // Now we take the element we're up to in the pool of possible
            // values (pool[i]), and find all the possible combinations
            // that could start with that value, using the pool we haven't
            // used yet, with k-1 iterations in. The value we're up to is then
            // whacked into that bit pattern to make it a proper K number of
            // combinations.

            // Base pattern showing the number we're starting with.
            let base_bit_pattern = ON << pool[i];

            // Get all the results for one less number of combinations and
            // not including the elements we've already used in this round.
            // Magic of recusrion makes this work as it bottoms out to individual
            // numbers above.
            //
            // I FUCKING LOVE RECURSION!
            let subresults = combo(&pool[i + 1..], k - 1);

            for element in subresults {
                // Now add on all those combinations with the starting number
                // we already have.
                result.push(base_bit_pattern | element);
            }
        }
    }

    result
}

/**
 * Struct that holds the bitmap of locations for candidate lines the represent
 * continuous horizontal or vertical lines. Used here as static data to loop over
 * later to reduce the need to repeat code for similiar patterns.
 */
#[derive(Debug)]
struct ConstantLine {
    // Pattern of boxes to match to check
    bit_pattern: u16,
    // Whether it's a horizontal or vertical match
    direction: Direction,
    // Which row/col it is
    index: usize,
}

const CONSTANT_LINES: [ConstantLine; 6] = [
    ConstantLine {
        bit_pattern: 0b0000000111,
        direction: Direction::HOR,
        index: 0,
    },
    ConstantLine {
        bit_pattern: 0b0000111000,
        direction: Direction::HOR,
        index: 1,
    },
    ConstantLine {
        bit_pattern: 0b0111000000,
        direction: Direction::HOR,
        index: 2,
    },
    ConstantLine {
        bit_pattern: 0b0001001001,
        direction: Direction::VER,
        index: 0,
    },
    ConstantLine {
        bit_pattern: 0b0010010010,
        direction: Direction::VER,
        index: 1,
    },
    ConstantLine {
        bit_pattern: 0b0100100100,
        direction: Direction::VER,
        index: 2,
    },
];

/**
 *
 * Solves a sudoku by applying the "candidate line" procedure. Each cell in the sudoku is
 * checked to find if there are any possible values in that cell exclusively in the same
 * row or column within the cell. If so this means that the specific possible value is
 * removed from the complete row or sudoku of the sudoku it is in.
 *
 * For instance if we find that the value '2' could only be in the mid row of the top-left
 * cell then we remove '2' as a possible value from the entire of the 2nd row of sudoku
 * in the top-mid and top-right cells
 */
pub fn candidate_line(sudoku: &mut Sudoku) {
    /*
     * Logic flow is:
     * for each cell
     *  for each possible value (1..9)
     *    build up a bitmap of each box the value is possible in
     *    Check for matches like (111000000b 000111000b 000000111b)
     *      if so remove value as poss from every box in this row
     *    Check for matches list (100100100b 010010010010b 001001001b)
     *      if so remove value as poss from every box in this column
     *
     * We can do this by constructing a series of bitmap masks that represent
     * linear runs within the cells (i.e. all the boxes in the top row or
     * the rightmost column) and then mask the actual values of the cell
     * against them to show that the value is in those areas, and then
     * an inverted mask to show that it's not also out side those areas.
     */
    for cell_idx in 0..9 {
        let cell = sudoku.cells[cell_idx];
        // Get bitmaps of possible values from 1-9, each array index has a bitmap
        // of possible values that location could be in this cell.
        let possibles = cell.bitmap_possibles();

        // Now for each value of 0-9 see if it matches one of our masks
        // for that value only being possible in a given row or column
        for candidate_value in 1..10 {
            let value_bitmap = possibles[candidate_value];

            // Skip those values where there's only one possible location
            // these are easier/faster to catch with out naive nethods and
            // complicate debugging.
            if (value_bitmap & (value_bitmap - 1)) == 0 {
                continue;
            }

            for checkline in CONSTANT_LINES.iter() {
                // Check that the value has values in the location bitmap
                // and nowhere else. If so we have a candidate line and can remove this
                // value from the rest of the sudoku that it lines up with.
                //
                // Essentially this shows that while we don't know where in this cell
                // a value will be, we know all possible lcoatiosn are in one row/col
                // so that we can remove that possibility from the other cells in the
                // same row/col in their boxes in line with this row.
                if (value_bitmap & checkline.bit_pattern) != 0
                    && (value_bitmap & !checkline.bit_pattern) == 0
                {
                    // We have found a candidate line! the candidate_value by matching
                    // the checkbitmap and only the check bitmap must be only in one row
                    // and/or column.
                    if checkline.direction == Direction::HOR {
                        // Confirmed we have a candidate line identified as a horizontal row, so
                        // need to find the index of the two cells next to this one first.
                        //
                        // First we get the indexes of all cells in this row, just take a div 3 of
                        // the index to find the leftmost box in the row, and then add 1 and 2 to
                        // find the next.
                        let left_idx = (cell_idx / 3) * 3;
                        let cells_in_row = vec![left_idx, left_idx + 1, left_idx + 2];

                        // Now loop over these cells, removing the candidate value from the appropriate
                        // row based on where the checkline structure says it is.
                        for cell_to_clean_idx in cells_in_row {
                            // Skip over the acual cell we found the index in
                            if cell_to_clean_idx != cell_idx {
                                // Function that actually removes said value from this specific row
                                sudoku.cells[cell_to_clean_idx]
                                    .rm_poss_from_row(candidate_value as u16, checkline.index);
                            }
                        }
                    } else if checkline.direction == Direction::VER {
                        // Confirmed we have a candidate line identified as a horizontal row, so
                        // need to find the index of the two cells next to this one first.
                        //
                        // First we get the indexes of all cells in this row, just take a div 3 of
                        // the index to find the leftmost box in the row, and then add 1 and 2 to
                        // find the next.
                        let top_idx = cell_idx % 3;
                        let cells_in_col = vec![top_idx, top_idx + 3, top_idx + 6];

                        // Now loop over these cells, removing the candidate value from the appropriate
                        // row based on where the checkline structure says it is.
                        for cell_to_clean_idx in cells_in_col {
                            // Skip the cell we're currently in of course.
                            if cell_to_clean_idx != cell_idx {
                                // Function that actually removes said value from this specific row
                                sudoku.cells[cell_to_clean_idx]
                                    .rm_poss_from_col(candidate_value as u16, checkline.index);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Inherit everything from up a level so we can run functions from there.
    use super::*;
    use crate::sk_box::BLANK_BOX;

    // Check that we can run a normalisation over 9 boxes that may not all
    // come from the same sudoku cell.
    #[test]
    fn test_last_value_line() {
        let mut line = Vec::new();

        let mut box1 = BLANK_BOX;
        let mut box2 = Box::from_val(2);
        let mut box3 = Box::from_val(3);
        let mut box4 = Box::from_val(4);
        let mut box5 = Box::from_val(5);
        let mut box6 = Box::from_val(6);
        let mut box7 = Box::from_val(7);
        let mut box8 = Box::from_val(8);
        let mut box9 = Box::from_val(9);

        line.push(&mut box1);
        line.push(&mut box2);
        line.push(&mut box3);
        line.push(&mut box4);
        line.push(&mut box5);
        line.push(&mut box6);
        line.push(&mut box7);
        line.push(&mut box8);
        line.push(&mut box9);

        // When we normalise a line from a random selection of boxes that are
        // interconnected we pass in reference to the 9 elements and they are modified
        // in place.
        single_position_array(line);

        assert!(box1.value == Some(1));
    }

    #[test]
    fn test_remove_possibles() {
        let mut line = Vec::new();

        let mut box1 = BLANK_BOX;
        let mut box2 = Box::from_val(2);
        let mut box3 = Box::from_val(3);
        let mut box4 = Box::from_val(4);
        let mut box5 = BLANK_BOX;
        let mut box6 = Box::from_val(6);
        let mut box7 = Box::from_val(7);
        let mut box8 = Box::from_val(8);
        let mut box9 = BLANK_BOX;

        line.push(&mut box1);
        line.push(&mut box2);
        line.push(&mut box3);
        line.push(&mut box4);
        line.push(&mut box5);
        line.push(&mut box6);
        line.push(&mut box7);
        line.push(&mut box8);
        line.push(&mut box9);

        // When we normalise a line from a random selection of boxes that are
        // interconnected we pass in reference to the 9 elements and they are modified
        // in place.
        single_position_array(line);

        assert!(box1.get_possibles() == [1, 5, 9]);
        assert!(box5.get_possibles() == [1, 5, 9]);
        assert!(box9.get_possibles() == [1, 5, 9]);
    }

    #[test]
    fn test_simple_combo() {
        let result = combo(&[1, 2, 3, 4], 2);
        // Combos we want to see are:
        // 1,2 / 1,3 / 1,4 / 2,3 / 2,4 / 3,4
        assert_eq!(result.len(), 6);
        assert!(result.contains(&(ON << 1 | ON << 2)));
        assert!(result.contains(&(ON << 1 | ON << 3)));
        assert!(result.contains(&(ON << 1 | ON << 4)));
        assert!(result.contains(&(ON << 2 | ON << 3)));
        assert!(result.contains(&(ON << 2 | ON << 4)));
        assert!(result.contains(&(ON << 3 | ON << 4)));
    }

    #[test]
    fn test_complex_combo() {
        let result = combo(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 4);
        // Combos we want to see are:
        // 1,2 / 1,3 / 1,4 / 2,3 / 2,4 / 3,4
        // Not checking everyone. There are 126 possible combinations of 4 objects from
        // a pool of 9 so justt check that.
        assert_eq!(result.len(), 126);
    }

    #[test]
    fn test_box_simplification_factor_2() {
        let mut line = Vec::new();

        let mut box1 = Box::from_possibles([1, 2].to_vec());
        let mut box2 = Box::from_possibles([1, 2].to_vec());
        let mut box3 = Box::from_possibles([1, 2, 3].to_vec());
        let mut box4 = Box::from_possibles([1, 2, 4].to_vec());
        let mut box5 = Box::from_possibles([1, 2, 5].to_vec());
        let mut box6 = Box::from_possibles([1, 2, 6].to_vec());
        let mut box7 = Box::from_possibles([1, 2, 7].to_vec());
        let mut box8 = Box::from_possibles([1, 2, 8].to_vec());
        let mut box9 = Box::from_possibles([1, 2, 9].to_vec());

        line.push(&mut box1);
        line.push(&mut box2);
        line.push(&mut box3);
        line.push(&mut box4);
        line.push(&mut box5);
        line.push(&mut box6);
        line.push(&mut box7);
        line.push(&mut box8);
        line.push(&mut box9);

        naked_set_array(line);

        assert_eq!(box3.get_possibles(), [3]);
        assert_eq!(box4.get_possibles(), [4]);
        assert_eq!(box5.get_possibles(), [5]);
        assert_eq!(box6.get_possibles(), [6]);
        assert_eq!(box7.get_possibles(), [7]);
        assert_eq!(box8.get_possibles(), [8]);
        assert_eq!(box9.get_possibles(), [9]);
    }

    #[test]
    fn test_box_simplification_factor_3() {
        let mut line = Vec::new();

        let mut box1 = Box::from_possibles([1, 2, 3].to_vec());
        let mut box2 = Box::from_possibles([1, 2, 3].to_vec());
        let mut box3 = Box::from_possibles([1, 2, 3].to_vec());
        let mut box4 = Box::from_possibles([1, 2, 4].to_vec());
        let mut box5 = Box::from_possibles([1, 2, 5].to_vec());
        let mut box6 = Box::from_possibles([1, 2, 6].to_vec());
        let mut box7 = Box::from_possibles([1, 2, 7].to_vec());
        let mut box8 = Box::from_possibles([1, 2, 8].to_vec());
        let mut box9 = Box::from_possibles([1, 2, 9].to_vec());

        line.push(&mut box1);
        line.push(&mut box2);
        line.push(&mut box3);
        line.push(&mut box4);
        line.push(&mut box5);
        line.push(&mut box6);
        line.push(&mut box7);
        line.push(&mut box8);
        line.push(&mut box9);

        naked_set_array(line);

        assert_eq!(box4.get_possibles(), [4]);
        assert_eq!(box5.get_possibles(), [5]);
        assert_eq!(box6.get_possibles(), [6]);
        assert_eq!(box7.get_possibles(), [7]);
        assert_eq!(box8.get_possibles(), [8]);
        assert_eq!(box9.get_possibles(), [9]);
    }

    #[test]
    fn test_box_simplification_factor_4() {
        let mut line = Vec::new();

        let mut box1 = Box::from_possibles([1, 2, 3, 4].to_vec());
        let mut box2 = Box::from_possibles([1, 2, 3, 4].to_vec());
        let mut box3 = Box::from_possibles([1, 2, 3, 4].to_vec());
        let mut box4 = Box::from_possibles([1, 2, 3, 4].to_vec());
        let mut box5 = Box::from_possibles([1, 2, 3, 5].to_vec());
        let mut box6 = Box::from_possibles([1, 3, 6].to_vec());
        let mut box7 = Box::from_possibles([1, 2, 7].to_vec());
        let mut box8 = Box::from_possibles([1, 3, 8].to_vec());
        let mut box9 = Box::from_possibles([1, 2, 9].to_vec());

        line.push(&mut box1);
        line.push(&mut box2);
        line.push(&mut box3);
        line.push(&mut box4);
        line.push(&mut box5);
        line.push(&mut box6);
        line.push(&mut box7);
        line.push(&mut box8);
        line.push(&mut box9);

        naked_set_array(line);

        assert_eq!(box5.get_possibles(), [5]);
        assert_eq!(box6.get_possibles(), [6]);
        assert_eq!(box7.get_possibles(), [7]);
        assert_eq!(box8.get_possibles(), [8]);
        assert_eq!(box9.get_possibles(), [9]);
    }

    #[test]
    fn test_box_simplification_factor_5() {
        let mut line = Vec::new();

        let mut box1 = Box::from_possibles([1, 2, 3, 4, 5].to_vec());
        let mut box2 = Box::from_possibles([1, 2, 3, 4, 5].to_vec());
        let mut box3 = Box::from_possibles([1, 2, 3, 4, 5].to_vec());
        let mut box4 = Box::from_possibles([1, 2, 3, 4, 5].to_vec());
        let mut box5 = Box::from_possibles([1, 2, 3, 4, 5].to_vec());
        let mut box6 = Box::from_possibles([1, 5, 6].to_vec());
        let mut box7 = Box::from_possibles([1, 2, 7].to_vec());
        let mut box8 = Box::from_possibles([5, 3, 8].to_vec());
        let mut box9 = Box::from_possibles([4, 2, 9].to_vec());

        line.push(&mut box1);
        line.push(&mut box2);
        line.push(&mut box3);
        line.push(&mut box4);
        line.push(&mut box5);
        line.push(&mut box6);
        line.push(&mut box7);
        line.push(&mut box8);
        line.push(&mut box9);

        naked_set_array(line);

        assert_eq!(box6.get_possibles(), [6]);
        assert_eq!(box7.get_possibles(), [7]);
        assert_eq!(box8.get_possibles(), [8]);
        assert_eq!(box9.get_possibles(), [9]);
    }

    #[test]
    pub fn candidate_line_test() {
        let mut sudoku = Sudoku::from_ss("test/candidate_line.ss".to_string()).unwrap();

        sudoku.pretty_print(None);
        candidate_line(&mut sudoku);
        sudoku.pretty_print(None);

        // Make sure the '456' possible in the middle of the mid top row have blocked out
        // both sides of it
        assert_eq!(
            sudoku.lookup(TOP_LFT, MID_LFT),
            Box::from_possibles([1, 2, 3, 7, 8, 9].to_vec())
        );

        assert_eq!(
            sudoku.lookup(TOP_LFT, MID_MID),
            Box::from_possibles([1, 2, 3, 7, 8, 9].to_vec())
        );

        assert_eq!(
            sudoku.lookup(TOP_LFT, MID_RHT),
            Box::from_possibles([1, 2, 3, 7, 8, 9].to_vec())
        );

        // Make sure the '789' possible in the middle of the mid right row have blocked out
        // above and below
        assert_eq!(
            sudoku.lookup(TOP_RHT, MID_LFT),
            Box::from_possibles([1, 2, 3, 7, 8, 9].to_vec())
        );

        assert_eq!(
            sudoku.lookup(TOP_RHT, MID_MID),
            Box::from_possibles([1, 2, 3].to_vec())
        );

        assert_eq!(
            sudoku.lookup(TOP_RHT, MID_RHT),
            Box::from_possibles([1, 2, 3, 7, 8, 9].to_vec())
        );

        assert_eq!(
            sudoku.lookup(TOP_RHT, TOP_MID),
            Box::from_possibles([1, 2, 3, 4, 5, 6].to_vec())
        );

        assert_eq!(
            sudoku.lookup(TOP_RHT, BOT_MID),
            Box::from_possibles([1, 2, 3, 4, 5, 6].to_vec())
        );

        // Make sure that 789 and removed as variables from the middle row of rht borttom right box
        assert_eq!(
            sudoku.lookup(BOT_RHT, TOP_MID),
            Box::from_possibles([1, 2, 3, 4, 5, 6].to_vec())
        );

        assert_eq!(
            sudoku.lookup(BOT_RHT, MID_MID),
            Box::from_possibles([1, 2, 3, 4, 5, 6].to_vec())
        );

        assert_eq!(
            sudoku.lookup(BOT_RHT, BOT_MID),
            Box::from_possibles([1, 2, 3, 4, 5, 6].to_vec())
        );
    }

    #[test]
    fn test_naive_row_solve() {
        let mut sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();

        // How does this not need a mut???
        let row = sudoku.get_row_mut(0);
        single_position_array(row);
        assert_eq!(sudoku.cells[TOP_LFT].boxes[TOP_LFT], Box::from_val(1));
    }

    #[test]
    fn test_single_position_sudoku_solve() {
        let mut sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();

        single_position(&mut sudoku);

        // Check that the top left most box got solved as it's the last in the row
        assert_eq!(sudoku.cells[TOP_LFT].boxes[TOP_LFT], Box::from_val(1));

        // Check that the center box got solved as it's the last in the column
        assert_eq!(sudoku.cells[MID_MID].boxes[MID_MID], Box::from_val(1));

        // Check that the bot right most box got solved as it's the last in the column
        assert_eq!(sudoku.cells[BOT_RHT].boxes[BOT_RHT], Box::from_val(1));
    }
}
