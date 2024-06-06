use crate::constants::*;
use crate::sk_box::*;
use std::fmt;
use std::io::stdout;
// use boxy::{Char, Weight};

use crossterm::{cursor::*, execute};

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

    /**
     *pretty_print
     *
     * Prints a nice version of a cell. See sudoku pretty_print for full context
     * Each box in the cell has it's own print function. This lays out a box lile
     *
     *    |    |
     *    |    |
     * ---+----+---
     *    |    |
     *    |    |
     * ---+----+---
     *    |    |
     *    |    |
     */
    pub fn pretty_print(&self, diff: Option<Cell>) {
        // Start by assuming we are at the top-left corner for cusor position anyway
        // then draw out the interior lines of the cell.

        // For each row of boxes in the cell start by drawing the outline
        for row_idx in 0..3 {
            // Draw the horizontal lines down the centre first, 3 times
            for _i in 0..3 {
                print!("   │   │");
                execute!(stdout(), MoveLeft(8), MoveDown(1)).ok();
            }

            // Aftger the first and second add an inbetween line
            if row_idx == 0 || row_idx == 1 {
                print!("───┼───┼───");
                execute!(stdout(), MoveLeft(11), MoveDown(1)).ok();
            }
        }
        // Now we are positioned below the box of our cell, inline with the first charachter
        // so we move back up to the top
        execute!(stdout(), MoveUp(11)).ok();

        // Now we print each of the 9 boxes in the cell to fill them in
        for row_idx in 0..3 {
            for cell_idx in 0..3 {
                let box_idx = (row_idx * 3) + cell_idx;
                match diff {
                    None => {
                        self.boxes[box_idx].pretty_print(None);
                    }
                    Some(diff_cell) => {
                        self.boxes[box_idx].pretty_print(Some(diff_cell.boxes[box_idx]));
                    }
                }
                execute!(stdout(), MoveRight(4)).ok();
            }
            execute!(stdout(), MoveLeft(12), MoveDown(4)).ok();
        }
        execute!(stdout(), MoveUp(12)).ok();
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
        array_check(self.boxes, true)
    }
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
            assert!(vals_found[x] ^ poss_found[x]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
