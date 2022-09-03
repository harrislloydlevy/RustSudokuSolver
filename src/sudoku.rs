use std::fs;
use std::io::stdout;
use std::io::BufRead;
use crate::constants::*;
use crate::sk_cell::*;
use crate::sk_box::*;
use crate::solvers::*;
// use boxy::{Char, Weight};

use crossterm::{
    execute,
    cursor::*
};

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
    pub cells: [Cell; 9]
}

const BLANK_SUDOKU:Sudoku = Sudoku {
    cells: [BLANK_CELL, BLANK_CELL, BLANK_CELL,
            BLANK_CELL, BLANK_CELL, BLANK_CELL,
            BLANK_CELL, BLANK_CELL, BLANK_CELL]
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
	pub fn get_row_mut<'a>(&'a mut self, row:usize) -> Vec<&'a mut Box> {
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
		let box_offset  = (row % 3) * 3;
		// println!("Row: {} CO: {} / BO: {}", row, cell_offset, box_offset);
		let mut cell_iter = self.cells.iter_mut();

		// Fast forward the cell iterator to the cell before the one we want
		// to read. Only if the iterator isn't already there for 0.
		if cell_offset > 0 {
			cell_iter.nth(cell_offset-1).unwrap();
		}

		for _x in 0..3 {
			let cell         = cell_iter.next().unwrap();

			// println!("Cell {}: {}", cell_offset+x, cell);
			let mut box_iter = cell.boxes.iter_mut();

			// Fast forward to the right row of the cell if necessary
			if box_offset > 0 {
				box_iter.nth(box_offset-1).unwrap();
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
	pub fn get_col_mut<'a>(&'a mut self, col:usize) -> Vec<&mut Box> {
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
		let box_offset  = col % 3;

		// println!("Col: {} CO: {} / BO: {}", col, cell_offset, box_offset);
		let mut cell_iter = self.cells.iter_mut();

		// Fast forward the cell iterator so the next cell it returns is the one
		// we want (0, 1, or 2).
		//
		// It would be "neater" to use get_nth to get the 3rd box every time to
		// scan 'vertically' down the array but it ends up messier code to deal
		// with getting the first box differently each time.
		if cell_offset > 0 {
			cell_iter.nth(cell_offset-1).unwrap();
		}

		for x in 0..3 {
			let cell         = cell_iter.next().unwrap();

			//println!("Cell {}: {}", cell_offset+(x*3), cell);
			let mut box_iter = cell.boxes.iter_mut();

			// Again we have to "fast forward, but now within the cell to the
			// right box.
			if box_offset > 0 {
				box_iter.nth(box_offset-1).unwrap();
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
	
	pub fn get_row(&self, row:usize) -> [Box; 9] {
		let cell_offset = (row / 3) * 3;
		let box_offset  = (row % 3) * 3;
		
		println!("{} / {}", cell_offset, box_offset);

		[
			self.cells[cell_offset  ].boxes[box_offset  ],
			self.cells[cell_offset  ].boxes[box_offset+1],
			self.cells[cell_offset  ].boxes[box_offset+2],
			self.cells[cell_offset+1].boxes[box_offset  ],
			self.cells[cell_offset+1].boxes[box_offset+1],
			self.cells[cell_offset+1].boxes[box_offset+2],
			self.cells[cell_offset+2].boxes[box_offset  ],
			self.cells[cell_offset+2].boxes[box_offset+1],
			self.cells[cell_offset+2].boxes[box_offset+2]
		]
	}

	/**
   *
   * get_box
   *
   * This function returns the box at a set cell and box index from the
   * sudoku. Removes the incovencine and bad prtactice of direclt accessing the
   * element.
	 *
	 * Note - doesn't return a ref, but a copy so cannot be used to modify sudoku!
	 */
	pub fn get_box(&self, cell_idx:usize, box_idx:usize) -> Box {
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
	pub fn get_cell(&self, cell_idx:usize) -> Cell {
		self.cells[cell_idx]
	}
    // This function creates a sudoku from a file. I don't knwo enough rust
    // yet to have it return a more generic error so just using io::Error
    //
    // File Format taken from Simple Sudoku
    pub fn from_ss(filename:String) -> Result<Sudoku, &'static str> {
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
            Err(error) => panic!("Problem opening the file: {:?}", error)
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

                // For windows and unix files this length could be different. Both should
                // start with a new line feed.
                if line.ends_with("\n\r") {
                  assert_eq!(length, 15); // Make sure there's enough data in line for all the rows
                  assert_eq!(line.pop(), Some('\r'));
                  assert_eq!(line.pop(), Some('\n'));
                } else {
                  // Otherwise assume a unix file
                  assert_eq!(length, 14); // Make sure there's enough data in line for all the rows
                  assert_eq!(line.pop(), Some('\n'));
                }


                // From 3 to 0 because we're going from right to left popping off end of the string.
                for cur_cel_col in (0..3).rev() {
                    // Read off the first '|'
                    assert_eq!(line.pop(), Some('|'));
                    for cur_box_col in (0..3).rev() {
                        let char = line.pop().expect("Expected box value, got EoL");
    
                        // Find the index of the cel and box to write into by multipleying
                        // row by 3. This matches our treatment of a linear 9 element array
                        // as a 3x3 array.
                        let cell_idx:usize = cur_cel_row * 3 + cur_cel_col;
                        let box_idx:usize = cur_box_row * 3 + cur_box_col;
    
                        // Initiate a new box to write into the Sudoku.
                        let value:Box;
    
                        // 
                        if char == '.' {
                            value = BLANK_BOX;
                        } else {
                            let digit = char.to_digit(10).expect("Expected number or '.'");
                            assert!(digit >= 1 && digit <= 9, "Expected a number between 1 and 9");
                            value = Box::from_val(digit as u8);
                        }
                        // To convert row and col to an index just times
                        // the row by 3. This matches our structure of a 9 element
                        // linear array represeting a 3x3 array
                        sudoku.
                            cells[cell_idx].
                            boxes[box_idx] = value;
                    }
                }
                line.clear();
            }
            // Check for a row of plain "---------" and read to the next line.
            // But if there's no lines left that's OK if we just read cell row 3
            reader.read_line(&mut line).expect("Could not read line.");
            line.clear();
        }
    
        return Ok(sudoku);
    }
    
    pub fn print_ss(&self) {
        println!("-----------");
        for cur_cel_row in 0..3 {
            
			for cur_box_row in 0..3 {
                println!("{}{}{}|{}{}{}|{}{}{}",
                         self.cells[cur_cel_row*3  ].boxes[cur_box_row*3  ],
                         self.cells[cur_cel_row*3  ].boxes[cur_box_row*3+1],
                         self.cells[cur_cel_row*3  ].boxes[cur_box_row*3+2],
                         self.cells[cur_cel_row*3+1].boxes[cur_box_row*3  ],
                         self.cells[cur_cel_row*3+1].boxes[cur_box_row*3+1],
                         self.cells[cur_cel_row*3+1].boxes[cur_box_row*3+2],
                         self.cells[cur_cel_row*3+2].boxes[cur_box_row*3  ],
                         self.cells[cur_cel_row*3+2].boxes[cur_box_row*3+1],
                         self.cells[cur_cel_row*3+2].boxes[cur_box_row*3+2]);
             }
             println!("-----------");
        }
    }

	/**
	 *
	 * solve
	 *
	 * Normalise the whole sudoku, resolving each cell, row and column.
     * 
     * TODO: This function not a complete solving problem. Simply runs one
     * round of the cheapest/fastests most basic sudoku solving techniques.
	 */
	pub fn solve(&mut self) {
	 	for cell in self.cells.iter_mut() {
			cell.solve()
		}

		for i in 0..9 {
			// How does this not need a mut???
            single_position_candidate(self.get_row_mut(i));
			naked_set(self.get_row_mut(i));
		}

		for i in 0..9 {
			// How does this not need a mut???
            single_position_candidate(self.get_col_mut(i));
			naked_set(self.get_col_mut(i));
		}
    }

    /**
     * pretty_print
     * 
     * Print to screen a nice version of the sudoku that shows actual
     * values and potential ones too. Every box is a 3x3 cell of numbers
     * showing potential and/or actual values so it's a pretty big box.
     */
    pub fn pretty_print(&self) {
        // This fucntion works by printing up the scaffolding of the
        // sudoku, then calling the cell pretty_print functions and
        // box pretty_print functions to draw in the provided location
        //
        // It's always assumed that the cursor is at the top-left corner
        // of the space to draw in.
        //
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

        // Now write the sides of the top boxes
        for cell_index in 0 .. 3 {
            if cell_index == 0 {
                // Drawing the first row so print the first line
              println!("╔═══════════╦═══════════╦═══════════╗");
            }

            // Draw the sides of the boxes
            for _i in 0.. 11 {
              println!("║           ║           ║           ║");
            }

            // If the very end of the sudoku draw the bottom, otherwise
            // draw a middle cross line
            
            if cell_index == 2 {
              print!("╚");
            } else {
              print!("╠");
            }

            for row_index in 0.. 3 {
              for _n in 0..11 {
                print!("═");
              }

              if row_index != 2 {
                if cell_index != 2 {
                  print!("╬");
                } else {
                  print!("╩");
                }
              }
            }

            if cell_index != 2 {
              print!("╣");
            } else {
              print!("╝");
            }
            println!("");
        }

        // Now we have to move through the 9 cells adjsuting the cursor to the top-left
        // each time and then calling the cell pretty print function to fill it out.
        //
        // First step is to get back to the top-left corner. We're at the line after
        // the whoel sudoku was printed now, and each cell is 11 boxes inside, plus the
        // bottom and intervening rows, 
        //
        // we assume each cell draw function sets the cursor back to the same position
        // as it was at the start.
        execute!(stdout(), MoveUp((11*3) + 3)).ok();

        for row_idx in 0 .. 3 {
            execute!(stdout(), MoveToColumn(1)).ok();
            for col_idx in 0 .. 3 {
                self.get_cell((row_idx*3) + col_idx).pretty_print();
                execute!(stdout(), MoveRight(12)).ok();
            }
            execute!(stdout(), MoveDown(12)).ok();
        }

        // End by moving back to start of colum as loop above should already
        // have ended up with us past the sudoku output.
        execute!(stdout(), MoveToColumn(1)).ok();
    }
}

#[cfg(test)]
mod tests {
	use super::*;

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

		    assert_eq!(*row1[0], Box::from_val(0));
		    assert_eq!(*row1[1], Box::from_val(0));
		    assert_eq!(*row1[2], Box::from_val(0));
		    assert_eq!(*row1[3], Box::from_val(2));
		    assert_eq!(*row1[4], Box::from_val(6));
		    assert_eq!(*row1[5], Box::from_val(0));
		    assert_eq!(*row1[6], Box::from_val(7));
		    assert_eq!(*row1[7], Box::from_val(0));
		    assert_eq!(*row1[8], Box::from_val(1));
			
			// Check updating works at end of test.
			// This is the top_right box of the top right cell
			*row1[8] = Box::from_val(9);
		}

		{
			let mut row3 = sudoku.get_row_mut(3);
			assert_eq!(*row3[0], Box::from_val(8));
			assert_eq!(*row3[1], Box::from_val(2));
			assert_eq!(*row3[2], BLANK_BOX);
			assert_eq!(*row3[3], Box::from_val(1));
			assert_eq!(*row3[4], BLANK_BOX);
			assert_eq!(*row3[5], BLANK_BOX);
			assert_eq!(*row3[6], BLANK_BOX);
			assert_eq!(*row3[7], Box::from_val(4));
			assert_eq!(*row3[8], BLANK_BOX);
	
			// Checked later at end of test
			// This is the top_mid box of the middle cell
			*row3[3] = Box::from_val(4);
		}
		{
		    let mut row7 = sudoku.get_row_mut(7);
		    assert_eq!(*row7[0], Box::from_val(0));
		    assert_eq!(*row7[1], Box::from_val(4));
		    assert_eq!(*row7[2], Box::from_val(0));
		    assert_eq!(*row7[3], Box::from_val(0));
		    assert_eq!(*row7[4], Box::from_val(5));
		    assert_eq!(*row7[5], Box::from_val(0));
			assert_eq!(*row7[6], Box::from_val(0));
			assert_eq!(*row7[7], Box::from_val(3));
			assert_eq!(*row7[8], Box::from_val(6));

			// Checked later at end of test
			*row7[8] = Box::from_val(2);
		}

		assert_eq!(sudoku.cells[TOP_RHT].boxes[TOP_RHT].value, Some(9));
		assert_eq!(sudoku.cells[MID_MID].boxes[TOP_LFT].value, Some(4));
		assert_eq!(sudoku.cells[BOT_RHT].boxes[MID_RHT].value, Some(2));
	}

	#[test]
	fn test_col_read_mut() {
		let mut sudoku = Sudoku::from_ss("test/simple.ss".to_string()).unwrap();

		{
		    let mut col1 = sudoku.get_col_mut(0);
		    assert_eq!(*col1[0], Box::from_val(0));
		    assert_eq!(*col1[1], Box::from_val(6));
		    assert_eq!(*col1[2], Box::from_val(1));
		    assert_eq!(*col1[3], Box::from_val(8));
		    assert_eq!(*col1[4], Box::from_val(0));
		    assert_eq!(*col1[5], Box::from_val(0));
		    assert_eq!(*col1[6], Box::from_val(0));
		    assert_eq!(*col1[7], Box::from_val(0));
		    assert_eq!(*col1[8], Box::from_val(7));

			// Set this to test it later.
			// THis is Bottom left box of the top left cell
			*col1[2] = Box::from_val(9);
		}

		{
			let mut col3 = sudoku.get_col_mut(3);
			assert_eq!(*col3[0], Box::from_val(2));
			assert_eq!(*col3[1], Box::from_val(0));
			assert_eq!(*col3[2], Box::from_val(0));
			assert_eq!(*col3[3], Box::from_val(1));
			assert_eq!(*col3[4], Box::from_val(6));
			assert_eq!(*col3[5], Box::from_val(0));
			assert_eq!(*col3[6], Box::from_val(3));
			assert_eq!(*col3[7], Box::from_val(0));
			assert_eq!(*col3[8], Box::from_val(0));
	
			// Checked later at end of test
			// This is the mid-left box of the centre cell.
			*col3[4] = Box::from_val(3);
		}

		{
		    let mut col7 = sudoku.get_col_mut(7);
		    assert_eq!(*col7[0], Box::from_val(0));
		    assert_eq!(*col7[1], Box::from_val(9));
		    assert_eq!(*col7[2], Box::from_val(0));
		    assert_eq!(*col7[3], Box::from_val(4));
		    assert_eq!(*col7[4], Box::from_val(0));
		    assert_eq!(*col7[5], Box::from_val(2));
			assert_eq!(*col7[6], Box::from_val(7));
			assert_eq!(*col7[7], Box::from_val(3));
			assert_eq!(*col7[8], Box::from_val(0));

			// Checked later at end of test
			// This is mid bottom call of the bottom right cell
			*col7[8] = Box::from_val(2);
		}

		// Now check that we can update values using these calls.
		assert_eq!(sudoku.cells[TOP_LFT].boxes[BOT_LFT].value, Some(9));
		assert_eq!(sudoku.cells[MID_MID].boxes[MID_LFT].value, Some(3));
		assert_eq!(sudoku.cells[BOT_RHT].boxes[BOT_MID].value, Some(2));
	}

	#[test]
	fn test_row_read() { 
		let sudoku = Sudoku::from_ss("test/simple.ss".to_string()).unwrap();

		let row1 = sudoku.get_row(0);
		assert_eq!(row1[0], Box::from_val(0));
		assert_eq!(row1[1], Box::from_val(0));
		assert_eq!(row1[2], Box::from_val(0));
		assert_eq!(row1[3], Box::from_val(2));
		assert_eq!(row1[4], Box::from_val(6));
		assert_eq!(row1[5], Box::from_val(0));
		assert_eq!(row1[6], Box::from_val(7));
		assert_eq!(row1[7], Box::from_val(0));
		assert_eq!(row1[8], Box::from_val(1));

		let row3 = sudoku.get_row(3);
		assert_eq!(row3[0], Box::from_val(8));
		assert_eq!(row3[1], Box::from_val(2));
		assert_eq!(row3[2], BLANK_BOX);
		assert_eq!(row3[3], Box::from_val(1));
		assert_eq!(row3[4], BLANK_BOX);
		assert_eq!(row3[5], BLANK_BOX);
		assert_eq!(row3[6], BLANK_BOX);
		assert_eq!(row3[7], Box::from_val(4));
		assert_eq!(row3[8], BLANK_BOX);

		let row7 = sudoku.get_row(7);
		assert_eq!(row7[0], Box::from_val(0));
		assert_eq!(row7[1], Box::from_val(4));
		assert_eq!(row7[2], Box::from_val(0));
		assert_eq!(row7[3], Box::from_val(0));
		assert_eq!(row7[4], Box::from_val(5));
		assert_eq!(row7[5], Box::from_val(0));
		assert_eq!(row7[6], Box::from_val(0));
		assert_eq!(row7[7], Box::from_val(3));
	}

	#[test]
	fn test_row_solve() {
		let mut sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();

		// How does this not need a mut???
		let row = sudoku.get_row_mut(0);
		single_position_candidate(row);
		assert_eq!(sudoku.cells[TOP_LFT].boxes[TOP_LFT], Box::from_val(1));
	}

	#[test]
	fn test_sudoku_solve() {
		let mut sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();

		sudoku.solve();

		// Check that the top left most box got solved as it's the last in the row
		assert_eq!(sudoku.cells[TOP_LFT].boxes[TOP_LFT], Box::from_val(1));

		// Check that the center box got solved as it's the last in the column
		assert_eq!(sudoku.cells[MID_MID].boxes[MID_MID], Box::from_val(1));

		// Check that the bot right most box got solved as it's the last in the column
		assert_eq!(sudoku.cells[BOT_RHT].boxes[BOT_RHT], Box::from_val(1));

	}

    #[test]
    fn test_sudoku_pretty_print() {
        // This is a shitty test - not sure how to test that console output matches a
        // expected outcome!
        //
        // TODO: Change test to comare outcome to a an expected file of output.
		    let mut sudoku = Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();

				// sudoku.cells[0].boxes[0].remove_possible_value(1);
				// sudoku.cells[0].boxes[0].remove_possible_value(5);
				// sudoku.cells[0].boxes[0].remove_possible_value(9);
				sudoku.solve();

        sudoku.pretty_print();


        assert!(true);
    }
}
