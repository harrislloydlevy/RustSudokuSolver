use crate::sk_box::*;
use std::fmt;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Cell {
    pub boxes: [Box; 9],
}

// Instantiating these structures from nothing can be time consuming so we pre-define a series of
// blank versions of each struct for convenience later here.

pub const BLANK_CELL: Cell = Cell {
    boxes: [
        BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX,
        BLANK_BOX,
    ],
};

impl fmt::Display for Cell {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("[").unwrap();
        for fmt_box in self.boxes.iter() {
            fmt_box.fmt(formatter).unwrap();
        }
        formatter.write_str("]")
    }
}

impl Cell {
    /**
     * get_mut
     *
     * Just like the get_row_mut and get_cell_mut functions of the overall
     * sudoku gets a vector of mutable boxes from the sudoku that solving functions
     * that work on sets of 9 boxes can run over.
     */
    pub fn get_mut<'a>(&'a mut self) -> Vec<&mut Box> {
        let mut result = Vec::new();
        for sk_box in self.boxes.iter_mut() {
            result.push(sk_box);
        }

        result
    }

    /**
     * set
     *
     * Set the cells values from a vec of integers.
     * 0 is taken to be blank.
     *
     * Used for intialising cells during sudoku construction so doesn't
     * set any possible values or consider whether cell is valid internally.
     */
    #[allow(dead_code)]
    pub fn set(&mut self, values: [u8; 9]) {
        assert_eq!(values.len(), 9);
        for x in 0..9 {
            let i = values[x];
            self.boxes[x] = Box::from_val(i);
        }
    }

    /**
     *
     * bitmap_possibles
     *
     * Get a 10 element array where each element is a bitmap of which boxes
     * in the cell as possibles for the value of the arrays index. I.e. index
     * 4 holds a bitmap of which boxes in the cell could possibly hold the
     * value 4.
     *
     * This gives us an array we can easily check how often a given value
     * is possible and in whch boxes which is useful for many solving techniques.
     *
     * For example in an cell where the values 1 and 3 where only possible in boxes
     * at index 4 and 5 and the array would have the value b000011000 and indexes 1 and 3
     */
    pub fn bitmap_possibles(&self) -> [u16; 10] {
        let mut result = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        // Cur_idx hold which of the boxes we are currently looking at
        // iterate over all of them tagging in their location index
        // as a bit pattern into each pattern they could possibly be.
        for idx in 0..9 {
            let cur_box = self.boxes[idx];

            // This is the bit pattern we tag in based on possible values.
            // It represents this boxes location, and we add it into
            // the bit pattern for each of the possible values.
            let bit_pattern = 1 << (idx);

            for poss_val in cur_box.get_possibles() {
                result[usize::from(poss_val)] = result[usize::from(poss_val)] | bit_pattern;
            }
        }
        result
    }

    /**
     * rm_poss_from_row
     *
     * Convenience function to remove any possibilities of a given value in a given
     * row in a cell.
     */
    pub fn rm_poss_from_row(&mut self, value: u16, row: usize) {
        let start_idx = row * 3;
        self.boxes[start_idx].remove_possible_value(value);
        self.boxes[start_idx + 1].remove_possible_value(value);
        self.boxes[start_idx + 2].remove_possible_value(value);
    }

    /**
     * rm_poss_from_col
     *
     * Convenience function to remove any possibilities of a given value in a given
     * row in a cell.
     */
    pub fn rm_poss_from_col(&mut self, value: u16, col: usize) {
        self.boxes[col].remove_possible_value(value);
        self.boxes[col + 3].remove_possible_value(value);
        self.boxes[col + 6].remove_possible_value(value);
    }

    pub fn solved(&self) -> bool {
        for sk_box in self.boxes {
            if !sk_box.solved() {
                return false;
            }
        }

        return true;
    }

    pub fn check(&self) {
        array_check(self.boxes, false)
    }
}

// Note - uses a mut type because most callers have an set of mut values to work with
// adn wecan't cast from mut to non-mut
//
// And to use this in the combo value later it needs to return an array, that
// has the format of being an array of possible values, 9 long, but with 0s
// at the end if less than 9 numbers are possible.
//
// For this purpose we pass the length with the array as well to allow the
// caller to generate a slice easily.
//
// I KNOW THIS LOOKS BAD! But it sort of makes sense for the caller and how it's
// used.
pub fn unsolved_values(input: &Vec<&mut Box>) -> ([u8; 9], usize) {
    let mut solved: [bool; 10] = [false; 10];
    let mut result: [u8; 9] = [0; 9];

    // Annoyed I can't think of a nice way to do this on one line with lambdas
    // that didn't involve multiple iterations of the same array
    input
        .iter()
        .filter(|x| x.solved())
        .for_each(|x| solved[x.get_value().expect("Not a real value") as usize] = true);

    let mut j: usize = 0;
    for i in 1..=9 {
        if solved[i] == false {
            result[j] = i as u8;
            j += 1;
        }
    }

    (result, j)
}

// Check if an array of 9 boxes is internally coherent.
// Doesn't reutrn anything, just asserts and crashes when somethign wrong
//
// When called with stricts makes sure possible values and actual line up
// correctly, when called without just makes sure that actual values do not
// repeat.
pub fn array_check(validate: [Box; 9], strict: bool) {
    for sk_box in validate {
        sk_box.check();
    }

    // Now check that each true value turns up only once.
    // Track this by keepign an array of what actual values we've found
    // Note the array are 10 items long to allow us to use the values 1-0 directly as indexes.
    let mut vals_found: [bool; 10] = [
        false, false, false, false, false, false, false, false, false, false,
    ];

    // We also want to make sure we've got all other possible values covered
    let mut poss_found: [bool; 10] = [
        false, false, false, false, false, false, false, false, false, false,
    ];

    // And tick them off as we go, making sure none of them turn up twice
    for sk_box in validate {
        match sk_box.value {
            None => {
                // If the box just has possibles, tick them off as being available
                // in the line.
                for x in 1..10 {
                    if sk_box.poss[x] {
                        poss_found[x] = true;
                    }
                }
            }
            Some(found_val) => {
                // If the box has a value tick it off as found, and make sure it
                // has not been seen before.
                let idx = usize::from(found_val);
                assert!(vals_found[idx] == false);
                vals_found[idx] = true;
            }
        }
    }

    if strict {
        // Now the validity test is to make sure that each value turns up as either found
        // or as a possible - but not both or neither!
        for x in 1..10 {
            //dbg!(x, vals_found[x], poss_found[x]);
            assert!(vals_found[x] ^ poss_found[x]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;
    use crate::solvers::*;
    use crate::sudoku::Sudoku;

    #[test]
    // Check that we can solve a box when there's only one value left.
    fn test_last_value_box() {
        let mut test_cell: Cell = Cell {
            boxes: [
                BLANK_BOX,
                Box::from_val(2),
                Box::from_val(3),
                Box::from_val(4),
                Box::from_val(5),
                Box::from_val(6),
                Box::from_val(7),
                Box::from_val(8),
                Box::from_val(9),
            ],
        };

        single_position_array(test_cell.get_mut());

        assert!(test_cell.boxes[TOP_LFT].value == Some(1));
    }

    #[test]
    fn test_set() {
        let mut test_cell: Cell = Cell {
            boxes: [
                BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX, BLANK_BOX,
                BLANK_BOX, BLANK_BOX,
            ],
        };

        test_cell.set(ARRAY_OF_9);
        assert!(test_cell.boxes[TOP_LFT].value == Some(1));
        assert!(test_cell.boxes[BOT_RHT].value == Some(9));
    }

    #[test]
    fn test_bitmap_possibles() {
        let sudoku = Sudoku::from_ss("test/simple.ss".to_string()).unwrap();

        let possibles = sudoku.cells[0].bitmap_possibles();

        // Value 1 shoudl only be possible in cell 7.
        assert_eq!(possibles[1], 1 << (7 - 1));

        // Value 2 shoudl only be possible in cells 1, 2, 3, 6, and 9
        assert_eq!(possibles[2], 0b100100111);

        // Value 3 shoudl only be possible in cells 1, 2, 3, 6, and 9
        assert_eq!(possibles[2], 0b100100111);
    }
}
