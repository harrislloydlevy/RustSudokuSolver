use crate::constants::*;
use crate::sk_box::*;
use crate::sk_cell::*;
use crate::solvers;
use std::fs;
use std::io::stdout;
use std::io::BufRead;
// use boxy::{Char, Weight};

use crossterm::{cursor::*, execute};

// Setup a data structure that represents a sudoku. It is made up on overall Sudouko, which
// consists of a 3x3 matrix of Cells, which are in turn a 3x3 matrix of Boxes. Each box has either
// a final value or notes the possible values.

// Each individual box consists of it's actual value (0 if unknown) and and array
// where each the index of a value holds a bool saying if it's possible. This is
// held as a ten element array for conveneince so that poss[value] will say whether
// that value is still possible.
//
// This is equivalent to an individual box in a paper sudoku noting either the final value, or
// small numbers around the edge of the box noting possible values.
//
// We are resisting the urge to squeeze this all into an u16 with the top bits used for the
// possible values and the bottom for the final. This project is to learn rust not play with
// bitwise operators.

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Sudoku {
    pub cells: [Cell; 9],
}

pub const BLANK_SUDOKU: Sudoku = Sudoku {
    cells: [
        BLANK_CELL, BLANK_CELL, BLANK_CELL, BLANK_CELL, BLANK_CELL, BLANK_CELL, BLANK_CELL,
        BLANK_CELL, BLANK_CELL,
    ],
};

impl Sudoku {
    /**
     * row_mut
     *
     * Returns a horizontal row across 3 cells. Takes an input 0-8 to which row to
     * return.
     *
     * Returns a mutable vec of references to the cells so as to be updated by
     * solving functions.
     *
     * This whole function is just fucked. It's way too complicated to get the
     * results out using iter_mut and nth functions on them across the item
     * and the weirdness in the function signature to set lifetimes on results
     * I don't entirely understand. If something breaks when we make this program
     * multi-threaded I'm pretty sure it will be here.
     */
    pub fn get_row_mut<'a>(&'a mut self, row: usize) -> Vec<&'a mut Box> {
        let mut result = Vec::new();

        // We will be iterating over 3 cells, and then 3 values
        // from each cell. We use the input row to select the
        // cell "offset" to get use to the right row of cells in the
        // sudoku, and then a box offset to get us to the right row
        // within each of those cells.
        //
        // Then we run a simple iteration over the cells, and then the boxes,
        // adding the offsets each time, to add our values in.

        // Divide 3 * 3 to set to either 0, 3, or 6.
        let cell_offset = (row / 3) * 3;

        // To find the offset within the cell mod by 3, then * 3
        // to be either 0, 3, or 6 within cell. Remember index from 0.
        let box_offset = (row % 3) * 3;
        let mut cell_iter = self.cells.iter_mut();

        // Fast forward the cell iterator to the cell before the one we want
        // to read. Only if the iterator isn't already there for 0.
        if cell_offset > 0 {
            cell_iter.nth(cell_offset - 1).unwrap();
        }

        for _x in 0..3 {
            let cell = cell_iter.next().unwrap();

            let mut box_iter = cell.boxes.iter_mut();

            // Fast forward to the right row of the cell if necessary
            if box_offset > 0 {
                box_iter.nth(box_offset - 1).unwrap();
            }

            for _y in 0..3 {
                let read_box = box_iter.next().unwrap();
                result.push(read_box);
            }
        }

        result
    }

    /**
     * get_col_mut
     *
     * Returns a vertical column across 3 cells. Takes an input 0-8 to which column to
     * return.
     *
     * Returns a mutable vec of references to the cells so as to be updated by
     * solving functions.
     *
     * This whole function is just fucked. It's way too complicated to get the
     * results out using iter_mut and nth functions on them across the item
     * and the weirdness in the function signature to set lifetimes on results
     * I don't entirely understand. If something breaks when we make this program
     * multi-threaded I'm pretty sure it will be here.
     */
    pub fn get_col_mut<'a>(&'a mut self, col: usize) -> Vec<&mut Box> {
        assert!(col <= 8);

        let mut result = Vec::new();

        // We will be iterating over 3 cells, and then 3 values
        // from each cell. We use the input row to select the
        // cell "offset" to get use to the right column of cells
        // (either 0, 1 or 2 offset) and then add 3 each time to
        // have a pattern of 0, 3, 6 /  1, 4, 7 / 2, 5, 8.
        //
        // Then the same patter to get the boxes within in column.
        //
        // Then we run a simple iteration over the cells, and then the boxes,
        // adding 3 each time.

        let cell_offset = col / 3;

        // To find the offset within the cell just mod by 3.
        let box_offset = col % 3;

        let mut cell_iter = self.cells.iter_mut();

        // Fast forward the cell iterator so the next cell it returns is the one
        // we want (0, 1, or 2).
        //
        // It would be "neater" to use get_nth to get the 3rd box every time to
        // scan 'vertically' down the array but it ends up messier code to deal
        // with getting the first box differently each time.
        if cell_offset > 0 {
            cell_iter.nth(cell_offset - 1).unwrap();
        }

        for x in 0..3 {
            let cell = cell_iter.next().unwrap();

            let mut box_iter = cell.boxes.iter_mut();

            // Again we have to "fast forward, but now within the cell to the
            // right box.
            if box_offset > 0 {
                box_iter.nth(box_offset - 1).unwrap();
            }

            for y in 0..3 {
                let read_box = box_iter.next().unwrap();
                result.push(read_box);

                // Now we skip the next two boxes unless we were reading the last value.
                if y < 2 {
                    box_iter.next().unwrap();
                    box_iter.next().unwrap();
                }
            }

            // Now we skip past the next two cells so that the start of the next loop will
            // read the next cell "down" on the array unless we're on the last read.
            if x < 2 {
                cell_iter.next().unwrap();
                cell_iter.next().unwrap();
            }
        }

        result
    }

    pub fn get_row(&self, row: usize) -> [Box; 9] {
        assert!(row < 9);
        let cell_offset = (row / 3) * 3;
        let box_offset = (row % 3) * 3;

        [
            self.lookup(cell_offset, box_offset),
            self.lookup(cell_offset, box_offset + 1),
            self.lookup(cell_offset, box_offset + 2),
            self.lookup(cell_offset + 1, box_offset),
            self.lookup(cell_offset + 1, box_offset + 1),
            self.lookup(cell_offset + 1, box_offset + 2),
            self.lookup(cell_offset + 2, box_offset),
            self.lookup(cell_offset + 2, box_offset + 1),
            self.lookup(cell_offset + 2, box_offset + 2),
        ]
    }

    pub fn get_col(&self, col: usize) -> [Box; 9] {
        assert!(col < 9);
        let cell_offset = col / 3;
        let box_offset = col % 3;

        [
            self.lookup(cell_offset, box_offset),
            self.lookup(cell_offset, box_offset + 3),
            self.lookup(cell_offset, box_offset + 6),
            self.lookup(cell_offset + 3, box_offset),
            self.lookup(cell_offset + 3, box_offset + 3),
            self.lookup(cell_offset + 3, box_offset + 6),
            self.lookup(cell_offset + 6, box_offset),
            self.lookup(cell_offset + 6, box_offset + 3),
            self.lookup(cell_offset + 6, box_offset + 6),
        ]
    }

    /**
     *
     * lookup
     *
     * This function returns the box at a set cell and box index from the
     * sudoku. Removes the incovencine and bad prtactice of direclt accessing the
     * element.
     *
     * Note - doesn't return a ref, but a copy so cannot be used to modify sudoku!
     */
    pub fn lookup(&self, cell_idx: usize, box_idx: usize) -> Box {
        self.cells[cell_idx].boxes[box_idx]
    }

    /**
     *
     * get_cell
     *
     * This function returns the cell at a given index
     * sudoku. Removes the incovencine and bad prtactice of direclt accessing the
     * element.
     */
    pub fn get_cell(&self, cell_idx: usize) -> Cell {
        self.cells[cell_idx]
    }

    // This function creates a sudoku from a file. I don't knwo enough rust
    // yet to have it return a more generic error so just using io::Error
    //
    // File Format taken from Simple Sudoku
    pub fn from_ss(filename: String) -> Result<Sudoku, &'static str> {
        // We expect to read a stream of numbers set out in the same
        // way a sudo would be printed on page, with "|" and "-" marks
        // used to break up the cells and the boxes in each cell just seperated by
        // spaces. Empty spaces are treated as blanks.
        //
        // Like below:
        // |.1.|012|012|
        // |345|345|345|
        // |678|..7|678|
        // -------------
        // |.12|912|912|
        // |345|345|345|
        // |678|678|678|
        // -------------
        // |.12|.12|.12|
        // |345|345|345|
        // |678|678|678|

        // Attempt to open the file
        let file = fs::File::open(filename);
        let file = match file {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };

        let mut reader = std::io::BufReader::new(file);

        // Instantiatie sudoku as blank
        let mut sudoku = BLANK_SUDOKU;
        // To read that stream into our more strutued 3- level tree we iterate
        // over:
        // 1. First over each of the 3 rows of cells in the sudoku (cur_cel_row)
        // 2. Then over each of 3 rows of boxes insides those cells (cur_box_row)
        //let cur_box_row = 0;

        // 3. The over the 3 cells that cross the row of numbers
        //let cur_cel_col = 0;

        // 4. Then we iterate over the boxes within that particular cell
        //let cur_box_col = 0;

        // These iterations then update the current cell, and the curernt box to
        // read the next value into.
        let mut line = String::new();
        for cur_cel_row in 0..3 {
            for cur_box_row in 0..3 {
                // Read a new line that crosses across all of the boxes.
                let length = reader.read_line(&mut line).expect("Could not read line");

                // Make sure there's enough data in line for all the row. May be 14 or 15 lines
                // depending on whether it's a unix or windows style text file.
                if length == 15 {
                    assert_eq!(line.pop(), Some('\n'));
                    assert_eq!(line.pop(), Some('\r'));
                } else if length == 14 {
                    assert_eq!(line.pop(), Some('\n'));
                } else {
                    assert!(false);
                }

                // Read charachters off from the RIGHT of the string using the pop
                // function. So first read off the \n and tehn continue right to
                // left.

                // From 3 to 0 because we're going from right to left popping off end of the string.
                for cur_cel_col in (0..3).rev() {
                    // Read off the first '|'
                    assert_eq!(line.pop(), Some('|'));
                    for cur_box_col in (0..3).rev() {
                        let char = line.pop().expect("Expected box value, got EoL");

                        // Find the index of the cel and box to write into by multipleying
                        // row by 3. This matches our treatment of a linear 9 element array
                        // as a 3x3 array.
                        let cell_idx: usize = cur_cel_row * 3 + cur_cel_col;
                        let box_idx: usize = cur_box_row * 3 + cur_box_col;

                        // To convert row and col to an index just times
                        // the row by 3. This matches our structure of a 9 element
                        // linear array represeting a 3x3 array
                        sudoku.cells[cell_idx].boxes[box_idx] = Self::char_to_box(char);
                    }
                }
                line.clear();
            }
            // Check for a row of plain "---------" and read to the next line.
            // But if there's no lines left that's OK if we just read cell row 3
            reader.read_line(&mut line).expect("Could not read line.");
            line.clear();
        }

        // Make sure that the "possibles" in each cell don't cross over with the
        // filled out values already in the cell.
        solvers::normalise(&mut sudoku);

        // Make sure the sudoku is well formed
        sudoku.check();

        return Ok(sudoku);
    }

    /**
     *
     * from_line
     *
     * Read a sudoku from a simple line definition, often found in files tha
     * contain lots of sudokus, one of each line.
     *
     * Just 81 numbers in a row for each value.
     */
    pub fn from_line(input: &String) -> Sudoku {
        let mut result: Sudoku = BLANK_SUDOKU;
        // I feel like a bad person for indexing from 1.
        let mut row = 1;
        let mut col = 1;

        assert!(input.len() == 81);

        for c in input.chars() {
            result.box_set(row, col, Self::char_to_box(c));
            row += 1;
            if row == 10 {
                row = 1;
                col += 1;
            }
        }
        assert!(row == 1);
        assert!(col == 10);

        solvers::normalise(&mut result);
        result
    }

    /**
     * Read every sudoku in a file and return them in a big array.
     */
    pub fn from_txt(filename: String) -> Vec<Sudoku> {
        let mut result = Vec::new();

        let file = fs::File::open(filename);
        let file = match file {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };

        let mut reader = std::io::BufReader::new(file);
        let mut line = String::new();

        while reader.read_line(&mut line).is_ok() {
            // Skip any lines less than 81 long.
            if line.len() == 84 {
                assert_eq!(line.pop(), Some('\n'));
                assert_eq!(line.pop(), Some('\r'));
            } else if line.len() == 82 {
                assert_eq!(line.pop(), Some('\n'));
            } else if line.len() == 0 {
                // Reading 0 length data shows we've reacehd the end of the file
                break;
            } else {
                // Any length by 81 is an error - nothing else allowed int eh file but sudokus.
                assert!(false);
            }

            result.push(Self::from_line(&line));
            line.clear();
        }

        result
    }

    fn char_to_box(char: char) -> Box {
        if char == '.' {
            BLANK_BOX
        } else {
            let digit = char.to_digit(10).expect("Expected number or '.'");
            assert!(
                digit >= 1 && digit <= 9,
                "Expected a number between 1 and 9"
            );
            Box::from_val(digit as u8)
        }
    }

    // TODO: Only public for testing. Shoudl work out how to avoid it.
    pub fn col_row_to_cell_idx(col: usize, row: usize) -> (usize, usize) {
        // Main job here is to turn the 1-9 absolute column and row
        // within the sudoku into a cell index (cells 1-9 within) and then box index (1-9 within)
        // Lots of mod 3 functions coming our way!

        // First get which cell it is. Simple row mod 3 will find if it's row 1, 2, or 3 and times
        // by 3 the result. Plus col mod 3 to find which one in the column of cells.
        let cell = ((col - 1) / 3) + (((row - 1) / 3) * 3);

        // Now to get the index within the cell.
        // Col within the cell is the cells absolute value mod 3
        let col_in_cell = (col - 1) % 3;
        let row_in_cell = (row - 1) % 3;
        let idx = col_in_cell + (row_in_cell * 3);

        (cell, idx)
    }

    pub fn box_set(&mut self, col: usize, row: usize, sk_box: Box) {
        let (cell, idx) = Self::col_row_to_cell_idx(col, row);

        self.cells[cell].boxes[idx] = sk_box;
    }

    pub fn get_c(&self, col: usize, row: usize) -> char {
        self.get_box(col, row).get_c()
    }

    pub fn get_box(&self, col: usize, row: usize) -> Box {
        let (cell, idx) = Self::col_row_to_cell_idx(col, row);

        self.cells[cell].boxes[idx]
    }

    pub fn print_ss(&self) {
        println!("-----------");
        for cur_cel_row in 0..3 {
            for cur_box_row in 0..3 {
                println!(
                    "{}{}{}|{}{}{}|{}{}{}",
                    self.cells[cur_cel_row * 3].boxes[cur_box_row * 3],
                    self.cells[cur_cel_row * 3].boxes[cur_box_row * 3 + 1],
                    self.cells[cur_cel_row * 3].boxes[cur_box_row * 3 + 2],
                    self.cells[cur_cel_row * 3 + 1].boxes[cur_box_row * 3],
                    self.cells[cur_cel_row * 3 + 1].boxes[cur_box_row * 3 + 1],
                    self.cells[cur_cel_row * 3 + 1].boxes[cur_box_row * 3 + 2],
                    self.cells[cur_cel_row * 3 + 2].boxes[cur_box_row * 3],
                    self.cells[cur_cel_row * 3 + 2].boxes[cur_box_row * 3 + 1],
                    self.cells[cur_cel_row * 3 + 2].boxes[cur_box_row * 3 + 2]
                );
            }
            println!("-----------");
        }
    }

    /**
     * pretty_print
     *
     * Print to screen a nice version of the sudoku that shows actual
     * values and potential ones too. Every box is a 3x3 cell of numbers
     * showing potential and/or actual values so it's a pretty big box.
     *
     * diff - If provided print cells that differ from this sudoku a
     *        different color
     */
    pub fn pretty_print(&self, diff: Option<Sudoku>, commentary: Option<String>) {
        // Each box is a 3x3 cell of text. If no confirmed values each
        // potential is showin in it's own position (1-9) and excluded
        // values are shown as a .
        //
        // If an actual value is found it's set in the middle of the box.
        //
        // Potential value are laid out like below:
        // 123     1.3
        // 456  or .5.  or 3
        // 789     .89
        //
        // These mean could be 1-9, could be 1,3,5,8, 9r 9, and is definitely 3
        // respectively.
        //
        // Boxes are laid out in a 3x3 matrix with single lines of text '|' or '-'
        // seperating them.
        //
        // Cells are then laid out in the overall sudoku with double lines between them.
        //
        // So this function just prints a shape like below, then moves the cursor around
        // to have teh cell pretty_print function do it's thing in each box:"
        // ╔════╦════╦════╗
        // ║    ║    ║    ║
        // ╠════╬════╬════╣
        // ║    ║    ║    ║
        // ╠════╬════╬════╣
        // ║    ║    ║    ║
        // ╚════╩════╩════╝

        /*
        These string constants used to get special cahrachters and then chars pasted
        into code as required.
        let weight = Weight::Doubled;
        let ul:String = Char::upper_left(weight).into_char().to_string();
        let ur:String = Char::upper_right(weight).into_char().to_string();
        let dt:String = Char::down_tee(weight).into_char().to_string();
        let hz:String = Char::horizontal(weight).into_char().to_string();
        let vr:String = Char::vertical(weight).into_char().to_string();
        */

        // First set our position to the top-left, and we can use that as the basis for everything
        // going forward.
        // Write the top line

        print!(
            "╔═══════════╦═══════════╦═══════════╗ {}",
            commentary.unwrap_or("".to_string())
        );
        println!("");
        // For each row of boxes in the sudoku we start a loop
        for row in 1..=9 {
            // If it's a "special" row we print some in-between decorations
            if row == 4 || row == 7 {
                println!("╠═══════════╬═══════════╬═══════════╣");
            }

            // Each row of the sudoku actually has 3 rows of text for printing out the sudoku box
            // in boxes of 3x3
            for val_row in 1..=3 {
                print!("║");
                for cell_col in 1..=3 {
                    // Each of the 3 columns of cells in the sudoku
                    for box_col in 1..=3 {
                        // Each of the 3 colums of boxes in the cell
                        for val_col in 1..=3 {
                            // Each of the 3 columns of possible values in the
                            // cell

                            let cell = ((cell_col - 1) * 3) + box_col;
                            // If there is a difference between the printed sudoku
                            // and the diff sudoku then print a special char
                            // to make the value red.
                            //
                            let orig_box = self.get_box(cell, row);
                            let mut changed = match diff {
                                None => false,
                                Some(compare) => orig_box != compare.get_box(cell, row),
                            };

                            if changed {
                                print!("\x1b[93m");
                            }
                            print!(
                                "{}",
                                self.get_box(cell, row)
                                    .get_pretty_c((val_row - 1) * 3 + val_col)
                            );
                            if changed {
                                print!("\x1b[0m")
                            }
                        }
                        if box_col != 3 {
                            print!("|");
                        }
                    }
                    print!("║");
                }
                println!("");
            }
            if row != 3 && row != 6 && row != 9 {
                println!("║---+---+---║---+---+---║---+---+---║");
            }
        }
        println!("╚═══════════╩═══════════╩═══════════╝");
    }

    // Check if the whole sudoku is solved.
    // simply check if all the cells are solved and only return true if none are unsolved
    pub fn solved(&self) -> bool {
        // Make sure it's consistent before we check it's solved.
        self.check();
        for cell in self.cells {
            if !cell.solved() {
                return false;
            }
        }

        return true;
    }

    // Check if the sudoku overall is still tip-top and internally consistent
    // Doesn't actually return anything, just triggers all the internal logical
    // consistency tests
    pub fn check(&self) {
        // Check all the cells are coherent.
        let mut _i: usize = 0;
        for cell in self.cells {
            cell.check();
            _i += 1;
        }

        // Checks each row for coherency
        for x in 0..9 {
            let row = self.get_row(x);
            array_check(row, false);
        }

        // Checks each col for coherency
        for x in 0..9 {
            let col = self.get_col(x);
            array_check(col, false);
        }
    }

    pub fn solve(&mut self) {
        self.pretty_print(None, Some("Solving".to_string()));
        self.check();
        let mut i = 0;
        while !self.solved() {
            let orig = *self;
            let mut prev = *self;

            // Try naive solving
            solvers::single_position(self);
            self.pretty_print(Some(prev), Some("Applied Single Position".to_string()));
            self.check();

            prev = *self;
            solvers::naked_set(self);
            self.pretty_print(Some(prev), Some("Applied Naked Set".to_string()));
            self.check();

            prev = *self;
            solvers::candidate_line(self);
            self.pretty_print(Some(prev), Some("Applied Candidate Line".to_string()));
            self.check();

            // If we made no progress at all over the whole last round - then we don't have the
            // abiliyt to solve this sudoku.
            if orig == *self {
                println!("Could not solve sudoku.");
                return;
            } else {
                println!("Going for round {}", i);
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers;

    #[test]
    fn test_blank_read() {
        // Test that reading and writing a sudoku works.
        // We test this by reading a sudoku from a test file, then writing out it back and and
        // ensuring the two values are the same.
        let result = Sudoku::from_ss("test/blank.ss".to_string()).unwrap();
        assert_eq!(result, BLANK_SUDOKU);
    }

    #[test]
    fn test_sparse_read() {
        // Test that reading and writing a sudoku works.
        // We test this by reading a sudoku from a test file, then writing out it back and and
        // ensuring the two values are the same.
        let result = Sudoku::from_ss("test/sparse.ss".to_string()).unwrap();
        assert_eq!(result.cells[TOP_LFT].boxes[TOP_MID].value, Some(1));
        assert_eq!(result.cells[TOP_LFT].boxes[MID_MID].value, Some(2));
        assert_eq!(result.cells[TOP_LFT].boxes[BOT_MID].value, Some(3));

        assert_eq!(result.cells[MID_MID].boxes[TOP_MID].value, Some(1));
        assert_eq!(result.cells[MID_MID].boxes[MID_MID].value, Some(2));
        assert_eq!(result.cells[MID_MID].boxes[BOT_MID].value, Some(3));

        assert_eq!(result.cells[BOT_RHT].boxes[TOP_MID].value, Some(1));
        assert_eq!(result.cells[BOT_RHT].boxes[MID_MID].value, Some(2));
        assert_eq!(result.cells[BOT_RHT].boxes[BOT_MID].value, Some(3));

        result.print_ss();
    }

    #[test]
    fn test_row_read_mut() {
        let mut sudoku = Sudoku::from_ss("test/simple.ss".to_string()).unwrap();

        {
            let mut row1 = sudoku.get_row_mut(0);

            assert!(row1[0].is_blank());
            assert!(row1[1].is_blank());
            assert!(row1[2].is_blank());
            assert_eq!(*row1[3], Box::from_val(2));
            assert_eq!(*row1[4], Box::from_val(6));
            assert!(row1[5].is_blank());
            assert_eq!(*row1[6], Box::from_val(7));
            assert!(row1[7].is_blank());
            assert_eq!(*row1[8], Box::from_val(1));

            // Check updating works at end of test.
            // This is the top_right box of the top right cell
            *row1[8] = Box::from_val(9);
        }

        {
            let mut row3 = sudoku.get_row_mut(3);
            assert_eq!(*row3[0], Box::from_val(8));
            assert_eq!(*row3[1], Box::from_val(2));
            assert!(row3[2].is_blank());
            assert_eq!(*row3[3], Box::from_val(1));
            assert!(row3[4].is_blank());
            assert!(row3[5].is_blank());
            assert!(row3[6].is_blank());
            assert_eq!(*row3[7], Box::from_val(4));
            assert!(row3[8].is_blank());

            // Checked later at end of test
            // This is the top_mid box of the middle cell
            *row3[3] = Box::from_val(4);
        }
        {
            let mut row7 = sudoku.get_row_mut(7);
            assert!(row7[0].is_blank());
            assert_eq!(*row7[1], Box::from_val(4));
            assert!(row7[2].is_blank());
            assert!(row7[3].is_blank());
            assert_eq!(*row7[4], Box::from_val(5));
            assert!(row7[5].is_blank());
            assert!(row7[6].is_blank());
            assert_eq!(*row7[7], Box::from_val(3));
            assert_eq!(*row7[8], Box::from_val(6));

            // Checked later at end of test
            *row7[8] = Box::from_val(2);
        }

        // Now check that we can update values using these calls.
        assert_eq!(sudoku.lookup(TOP_RHT, TOP_RHT).value, Some(9));
        assert_eq!(sudoku.lookup(MID_MID, TOP_LFT).value, Some(4));
        assert_eq!(sudoku.lookup(BOT_RHT, MID_RHT).value, Some(2));
    }

    #[test]
    fn test_col_read_mut() {
        let mut sudoku = Sudoku::from_ss("test/simple.ss".to_string()).unwrap();

        {
            let mut col1 = sudoku.get_col_mut(0);
            assert!(col1[0].is_blank());
            assert_eq!(*col1[1], Box::from_val(6));
            assert_eq!(*col1[2], Box::from_val(1));
            assert_eq!(*col1[3], Box::from_val(8));
            assert!(col1[4].is_blank());
            assert!(col1[5].is_blank());
            assert!(col1[6].is_blank());
            assert!(col1[7].is_blank());
            assert_eq!(*col1[8], Box::from_val(7));

            // Set this to test it later.
            // THis is Bottom left box of the top left cell
            *col1[2] = Box::from_val(9);
        }

        {
            let mut col3 = sudoku.get_col_mut(3);
            assert_eq!(*col3[0], Box::from_val(2));
            assert_eq!(*col3[3], Box::from_val(1));
            assert_eq!(*col3[4], Box::from_val(6));
            assert_eq!(*col3[6], Box::from_val(3));

            assert!(col3[1].is_blank());
            assert!(col3[2].is_blank());
            assert!(col3[5].is_blank());
            assert!(col3[7].is_blank());
            assert!(col3[8].is_blank());

            // Checked later at end of test
            // This is the mid-left box of the centre cell.
            *col3[4] = Box::from_val(3);
        }

        {
            let mut col7 = sudoku.get_col_mut(7);
            assert_eq!(*col7[1], Box::from_val(9));
            assert_eq!(*col7[3], Box::from_val(4));
            assert_eq!(*col7[5], Box::from_val(2));
            assert_eq!(*col7[6], Box::from_val(7));
            assert_eq!(*col7[7], Box::from_val(3));
            assert!(col7[0].is_blank());
            assert!(col7[2].is_blank());
            assert!(col7[4].is_blank());
            assert!(col7[8].is_blank());

            // Checked later at end of test
            // This is mid bottom call of the bottom right cell
            *col7[8] = Box::from_val(2);
        }

        // Now check that we can update values using these calls.
        assert_eq!(sudoku.lookup(TOP_LFT, BOT_LFT).value, Some(9));
        assert_eq!(sudoku.lookup(MID_MID, MID_LFT).value, Some(3));
        assert_eq!(sudoku.lookup(BOT_RHT, BOT_MID).value, Some(2));
    }

    #[test]
    fn test_row_read() {
        let sudoku = Sudoku::from_ss("test/simple.ss".to_string()).unwrap();

        let row1 = sudoku.get_row(0);
        assert!(row1[0].is_blank());
        assert!(row1[1].is_blank());
        assert!(row1[2].is_blank());
        assert_eq!(row1[3], Box::from_val(2));
        assert_eq!(row1[4], Box::from_val(6));
        assert!(row1[5].is_blank());
        assert_eq!(row1[6], Box::from_val(7));
        assert!(row1[7].is_blank());
        assert_eq!(row1[8], Box::from_val(1));

        let row3 = sudoku.get_row(3);
        assert_eq!(row3[0], Box::from_val(8));
        assert_eq!(row3[1], Box::from_val(2));
        assert!(row3[2].is_blank());
        assert_eq!(row3[3], Box::from_val(1));
        assert!(row3[4].is_blank());
        assert!(row3[5].is_blank());
        assert!(row3[6].is_blank());
        assert_eq!(row3[7], Box::from_val(4));
        assert!(row3[8].is_blank());

        let row7 = sudoku.get_row(7);
        assert!(row7[0].is_blank());
        assert_eq!(row7[1], Box::from_val(4));
        assert!(row7[2].is_blank());
        assert!(row7[3].is_blank());
        assert_eq!(row7[4], Box::from_val(5));
        assert!(row7[5].is_blank());
        assert!(row7[6].is_blank());
        assert_eq!(row7[7], Box::from_val(3));
    }

    #[test]
    fn test_sudoku_pretty_print_single() {
        // This is a shitty test - not sure how to test that console output matches a
        // expected outcome!
        //
        // TODO: Change test to compare outcome to a an expected file of output.
        let mut sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();

        // sudoku.cells[0].boxes[0].remove_possible_value(1);
        // sudoku.cells[0].boxes[0].remove_possible_value(5);
        // sudoku.cells[0].boxes[0].remove_possible_value(9);
        solvers::single_position(&mut sudoku);

        assert!(true);
    }

    #[test]
    fn test_sudoku_pretty_print_compare() {
        // This is a shitty test - not sure how to test that console output matches a
        // expected outcome!
        //
        // TODO: Change test to comare outcome to a an expected file of output.
        let unsolved = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();
        let mut solved = unsolved;
        solvers::single_position(&mut solved);

        assert!(true);
    }

    #[test]
    fn test_solved() {
        let sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();
        assert!(!sudoku.solved());

        let sudoku = Sudoku::from_ss("test/solved.ss".to_string()).unwrap();
        assert!(sudoku.solved());
    }

    #[test]
    fn test_from_line() {
        let sud_line = "\
            6.2.5....\
            .....4.3.\
            .........\
            43...8...\
            .1....2..\
            ......7..\
            5..27....\
            .......81\
            ...6.....";

        let sudoku = Sudoku::from_line(&sud_line.to_string());
        assert_eq!(sudoku.get_c(1, 1), '6');
        assert_eq!(sudoku.get_c(5, 1), '5');
        assert_eq!(sudoku.get_c(9, 9), '.');
    }

    #[test]
    fn test_col_row_cell_idx() {
        assert_eq!(Sudoku::col_row_to_cell_idx(1, 1), (0, 0));
        assert_eq!(Sudoku::col_row_to_cell_idx(2, 5), (3, 4));
        assert_eq!(Sudoku::col_row_to_cell_idx(5, 5), (4, 4));
        assert_eq!(Sudoku::col_row_to_cell_idx(7, 3), (2, 6));
        assert_eq!(Sudoku::col_row_to_cell_idx(7, 5), (5, 3));
        assert_eq!(Sudoku::col_row_to_cell_idx(8, 7), (8, 1));
        assert_eq!(Sudoku::col_row_to_cell_idx(9, 9), (8, 8));

        //for row in 1..10 {
        //    for col in 1..10 {
        //        let (cell, idx) = Sudoku::col_row_to_cell_idx(col, row);
        //        println!("[{},{}] => [{}][{}]", col, row, cell, idx);
        //    }
        //}
    }

    #[test]
    fn test_read_txt_file() {
        let result = Sudoku::from_txt("test/top95.txt".to_string());

        assert_eq!(result.len(), 95);
    }

    #[test]
    fn test_solve_txt_file() {
        let result = Sudoku::from_txt("test/top95.txt".to_string());

        assert_eq!(result.len(), 95);

        for mut sudoku in result {
            sudoku.solve();
            assert!(sudoku.solved());
        }
    }

    #[test]
    fn test_solvable() {
        let result = Sudoku::from_txt("test/solvable.txt".to_string());
        let mut i = 1;

        for mut sudoku in result {
            sudoku.solve();
            assert!(sudoku.solved());
            println!("Solved {} Sudokus!", i);
            i += 1;
        }
    }
}
