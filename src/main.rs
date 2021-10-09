// TODO: Refactor out each 'class' to own file
// TODO: Move each solving function out to it's own library and make each generic signature
// TODO: Add new solving function where if 'x' boxes all can _only_ be the same 'x' values then
//       any other boxes in their row/cell can't be those values
// TODO: Conitinual solving over the sudoku trying new solving fucntions until done/stuck
// TODO: 'reduce' fucntuon to merge two sudokus of possibles and solving as 'map' function
//


use std::fs;
use std::io;
use std::fmt;
use std::io::BufRead;


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
struct Box {
    value: Option<u8>,
    poss: [bool; 10]
}

const ON:u16  = 1;
const OFF:u16 = 0;

// Consts to easily get the index of a given positions in a 3x3 array that's stored
// as an array. Implmetned as usize as they are used to lookup arrays.
const TOP_LFT:usize = 0;
const TOP_MID:usize = 1;
const TOP_RHT:usize = 2;
const MID_LFT:usize = 3;
const MID_MID:usize = 4;
const MID_RHT:usize = 5;
const BOT_LFT:usize = 6;
const BOT_MID:usize = 7;
const BOT_RHT:usize = 8;

const BLANK_BOX:Box = Box {
    value: None,
    poss: [false, true, true, true, true, true, true, true, true, true]
};

const BOX_EMPTY_POSS: [bool; 10] = [false, false, false, false, false, false, false, false, false, false];

impl fmt::Display for Box {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            Some(x) => formatter.write_fmt(format_args!("{}", x)),
            None    => formatter.write_str(".")
        }
    }
}

impl Box {
    /**
     * box_value
     *
     * Return a box of a particular value.
	 *
	 * Returns a blank box is passed 0. I know it seems dumb, but it made
	 * writing up the tests easier in parts.
     */
    fn from_val(x:u8) -> Box {
        let mut result = BLANK_BOX;
		if x != 0 {
			result.set_val(x);
		}
   
        result
    }

	/**
     * set_val
	 *
	 * Set teh value of an existing box, including the possible valyues.
	 */
	fn set_val(&mut self, x:u8)	{
		self.value = Some(x);
		self.poss = BOX_EMPTY_POSS;
		self.poss[x as usize] = true;
	}

	/**
     * from_possibles
	 *
 	 * Create a new box without a known value, from with a known set of possible values.
	 */
	fn from_possibles(possibles:Vec<u8>) -> Box {
		let mut new_box = BLANK_BOX;
		new_box.set_possibles(possibles);
		new_box
	}

	/**
	 * remove_possible_bits
	 *
	 * Based on a bitmap further restrict the box removing any values not makred as possible
	 * This doesn't add any new possibilities if they are possible in the map, just removes.
	 *
	 * Useful when an algorithim has shown values as not possible, but doesn't say anything
	 * about whic ones are possible
	 */ 
	fn remove_possible_bits(&mut self, possible_bits:u16) {
		let curr_poss = self.get_possibles_bits();
		self.set_possibles_bits(curr_poss & possible_bits);
		
	}

	/**
     * from_possibles_bits
	 *
 	 * Create a new box without a known value, from with a known set of possible values.
	 */
	fn from_possibles_bits(possibles:u16) -> Box {
		let mut new_box = BLANK_BOX;
		new_box.set_possibles_bits(possibles);
		new_box
	}

	/**
 	 * set_possiblities
	 *
	 * Set what the new possible values of this box could be. From list of u8s.
	 *
	 * Note that setting a *single* possibility implicitly sets that possibility
	 * as the value for this box.
	 */
	fn set_possibles(&mut self, possibles:Vec<u8>) {
		assert!(possibles.len() > 0);
		assert!(possibles.len() <= 9);
		match possibles.len() {
			// If just a single value revert to setting that value as if it was a flat out set.
			1 => self.set_val(possibles[0]),
			// If a list set us back to 0 and set true for only those values we're given.
			_ => {
				// Should not have value know if we're setting possibles! Can't go backwards.
				assert!(self.value == None);
				self.poss = BOX_EMPTY_POSS;
				for x in possibles {
					self.poss[x as usize] = true;
				}
			}
		}
	}
    
	/**
 	 * set_possiblities_bits
	 *
	 * Set what the new possible values of this box could be. From bit pattern.
	 * As I'm lazy and don't want to deal with a ton of "off by one" bugs the bit
	 * pattern starts from index 1 so to set or clear a possibility you set the
	 * 1 << value where value is from 1 to 9.
	 *
	 * Note that setting a *single* possibility implicitly sets that possibility
	 * as the value for this box.
	 */
	fn set_possibles_bits(&mut self, possibles:u16) {
		// Can never have no options.
		assert!(possibles != 0);
		// Ensure no bits set above the 9th position by checking bitmask
		// against 01111111110;
		assert!((possibles & 0b1111111110) == possibles);

		// Check if there is only a single bit set 
		if possibles == possibles & (!(possibles-1)) {
			// Unforunately doing a match on (1 >> 1) doesn't work so we need to
			// check for exact bit patterns.
			match possibles {
				        0b10 => self.set_val(1),
				       0b100 => self.set_val(2),
				      0b1000 => self.set_val(3),
				     0b10000 => self.set_val(4),
				    0b100000 => self.set_val(5),
				   0b1000000 => self.set_val(6),
				  0b10000000 => self.set_val(7),
				 0b100000000 => self.set_val(8),
				0b1000000000 => self.set_val(9),
				_            => assert!(false)
			}
		} else {
			// Otherwise there are multiple possible values here. Iterate over them
			let mut n = 0;
			while n <= 9 {
				self.poss[n] = (possibles >> n & 0b1) == 0b1;
				n = n+1;
			}
		}
	}

	/**
	 *
	 * get_possible_bits
	 *
	 * Get a list of possible values as a bit mask.
	 *
	 */
	fn get_possibles_bits(&self) -> u16 {
		let mut result:u16 = 0;

		for x in 1..9 {
			if self.poss[x] {
				result |= ON << x;
			}
		}

		result
	}




    /**
     * check
     * 
     * Check that a box is internally consistent and in a "good" state that doesn't represent and
     * internal inconsistency.
     * 
     * Doesn't retrun anything just asserts if the box is invalid.
     */
    fn check(self: Box) {
        match self.value {
            Some(x) => {
            	// If we have a confirmed value just check that it's between 1-9 and the possibles
            	// values array matches the confirmed value.
    			assert!(x >= 1);
    			assert!(x <= 9);
    
    			// As we do sometimes use the "possibles array make sure it shows the only possible
    			// value in this box is it's actual value.
        		let mut poss_values = [false, false, false, false, false, false, false, false, false, false];
    			poss_values[x as usize] = true;
    			
    			assert!(self.poss == poss_values);
            },
            None => {
    			// Check with no confirmed value is that "0" is not a possible value.
    			assert!(self.poss[0] == false);

				// Check that there is at least one index of the array of possible values that is positive.
				let mut found_true = false;
				for x in self.poss.iter() {
					found_true |= x;
					println!("{} / {}", x, found_true);
				}
				assert!(found_true);
            }
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Cell {
    boxes: [Box; 9]
}

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
	// Run "normalization" which is the simplest form of solving any 9 element sudoku
	// over this particular cell.
    fn normalise(&mut self) {
		// We can't just call normalise_boxes direclty as that fucntion expect an array
		// of references to boxes instead of an array of boxes. So we have to pass
		// in an array of refs instead.

		let mut vec = Vec::new();
		for x in self.boxes.iter_mut() {
			vec.push(x);
		}

		normalise_boxes(vec);
	}
}

/**
 * normalise_boxes
 *
 * This is the dumbest solving algorithim there is.
 * 
 * It looks over a list of 9
 * boxes and for any boxes that are solved, it removes the solved boxes from
 * thee list of possible for the other boxes. If one of the boxes now only
 * has a single possible value then it's set as solved to that value.
 */
fn normalise_boxes(mut boxes: Vec<&mut Box>) {
	// pos_vals is the bit mask of still possible values in this set of interlinked
	// boxes.
	//
	// Each soved valuewill zero out it's own value in the mask so as to mark it as
	// not possible in the unsovled values in the cell.
	//
	// We 
	let mut poss_vals:u16 = 0b1111111110;
	for x in boxes.iter() {
		// If we have an actual value we blank out that possible value from the map
		// otherwise ignore the uncionfirmed values.
		match x.value {
			// mask it off against the inverse of the found value.
			Some(confirmed_value) => {
				poss_vals &= !( ON << confirmed_value)
			},
			None => ()
		};
	}

	// Now in poss_vals we have an bitmap that represents all the values that nothing
	// else can be. So we apply that to each of the values in the 9 array
	// set that are still looking for a value.
	for unsolved_box in boxes.iter_mut() {
		// If we have an unconfirmed values remove the possibilities foumd, otherwise
		// for solved boxes we just skip over.
		match unsolved_box.value {
			Some(_unused) => {},
			None => unsolved_box.remove_possible_bits(poss_vals)
		}
	}
}

#[derive(PartialEq, Debug)]
struct Sudoku {
    cells: [Cell; 9]
}

// Instantiating these structures from nothing can be time consuming so we pre-define a series of
// blank versions of each struct for convenience later here.

const BLANK_CELL:Cell = Cell {
    boxes: [BLANK_BOX, BLANK_BOX, BLANK_BOX,
            BLANK_BOX, BLANK_BOX, BLANK_BOX,
            BLANK_BOX, BLANK_BOX, BLANK_BOX]
};

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
	fn get_row_mut<'a>(&'a mut self, row:usize) -> Vec<&'a mut Box> {
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
		println!("Row: {} CO: {} / BO: {}", row, cell_offset, box_offset);
		let mut cell_iter = self.cells.iter_mut();

		// Fast forward the cell iterator to the cell before the one we want
		// to read. Only if the iterator isn't already there for 0.
		if cell_offset > 0 {
			cell_iter.nth(cell_offset-1).unwrap();
		}

		for x in 0..3 {
			let cell         = cell_iter.next().unwrap();

			println!("Cell {}: {}", cell_offset+x, cell);
			let mut box_iter = cell.boxes.iter_mut();

			// Fast forward to the right row of the cell if necessary
			if box_offset > 0 {
				box_iter.nth(box_offset-1).unwrap();
			}
			
			for y in 0..3 {
				let read_box = box_iter.next().unwrap();
				println!("Reading {} / {} - {}", cell_offset+x, box_offset+y, read_box);
				result.push(read_box);
			}
		}

		result
	}

	/**
     * col_mut
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
	fn get_col_mut<'a>(&'a mut self, col:usize) -> Vec<&mut Box> {
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

		println!("Col: {} CO: {} / BO: {}", col, cell_offset, box_offset);
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
				println!("Reading {} / {} - {}", cell_offset+x, box_offset+y, read_box);
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
	
	fn get_row(&self, row:usize) -> [Box; 9] {
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

    // This function creates a sudoku from a file. I don't knwo enough rust
    // yet to have it return a more generic error so just using io::Error
    //
    // File Format taken from Simple Sudoku
    fn from_ss(filename:String) -> Result<Sudoku, io::Error> {
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
                assert_eq!(length, 15); // Make sure there's enough data in line for all the rows
    
                // Read charachters off from the RIGHT of the string using the pop
                // function. So first read off the \n and tehn continue right to
                // left.
                assert_eq!(line.pop(), Some('\n'));
                assert_eq!(line.pop(), Some('\r'));
    
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
    
    fn print_ss(&self) {
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
	 * Normalise the whole sudoku, resolving each cell, row and column until
	 * a whole round has gone through with no changes to the sudoku.
	 */
	fn solve(&mut self) {
	 	for cell in self.cells.iter_mut() {
			cell.normalise()
		}

		for i in 0..9 {
			// How does this not need a mut???
			let row = self.get_row_mut(i);
			normalise_boxes(row);
		}

		for i in 0..9 {
			// How does this not need a mut???
			let col = self.get_col_mut(i);
			normalise_boxes(col);
		}
    }
}

fn main() {
    println!("Hellow World");
}

#[cfg(test)]
mod tests {
    // Inherit everything from up a level so we can run functions from there.
    
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
	fn test_ok_value_box() {

		// Ensure box with a single value passes
		let ok_value_box = Box::from_val(2);
		println!("OK BOX: {:?}", ok_value_box);
		ok_value_box.check();
	}

	#[test]
	#[should_panic]
	// Checks that a box with no possible values will fail
	fn test_no_poss_box() {
		let mut ok_no_value = BLANK_BOX;
		ok_no_value.poss = BOX_EMPTY_POSS;
			

		// This box has no value so should fail it's check.
		ok_no_value.check();
	}

	#[test]
	#[should_panic]
	// Checks that values outside of the 0-9 range fail
	fn test_bad_value_box() {
		let bad_value =
			Box {
				value: Some(11),
				poss: [false,false,false,false,false,false,false,false,false,false]
			};

		// This box has no value so should pass all it's test.
		bad_value.check();
	}

	#[test]
	#[should_panic]
	// Checks for a box with a set value, but a possibles array that doesn't match.
	fn test_bad_possibles_box() {
		let bad_value =
			Box {
				value: Some(4),
				poss: [false,true,false,false,false,true,false,false,false,false]
			};

		bad_value.check();
	}

	#[test]
	#[should_panic]
	// Checks that a box with no possibilities fails.
	fn test_has_possibles_box() {
		let bad_value =
			Box {
				value: None, 
				poss: [false,false,false,false,false,false,false,false,false,false]
			};

		bad_value.check();
	}

	#[test]
	// Check that a boxes methods for updating and reading values stay consistent.
	fn test_value_set_and_read() {
		let setter = Box::from_possibles(vec![1,4,7]);

		assert!(setter.poss ==
			[false, true, false, false, true, false, false, true, false, false]);

		// Now we do the same but with a bit pattern.
		let bit_pattern:u16 = ON << 1 | OFF << 2 | ON << 4 | ON <<7;
		let setter = Box::from_possibles_bits(bit_pattern);

		assert_eq!(bit_pattern, setter.get_possibles_bits());

		assert!(setter.poss ==
			[false, true, false, false, true, false, false, true, false, false]);

	}

	#[test]
	// Check that we can solve a box when there's only one value left.
	fn test_last_value_box() {
		let mut test_cell:Cell = Cell {
			boxes:[ BLANK_BOX,        Box::from_val(2), Box::from_val(3),
                    Box::from_val(4), Box::from_val(5), Box::from_val(6),		
                    Box::from_val(7), Box::from_val(8), Box::from_val(9)]
		};
 
		test_cell.normalise();

		assert!(test_cell.boxes[TOP_LFT].value == Some(1));
	}

	#[test]
	// Check that we can run a normalisation over 9 boxes that may not all
	// come from the same sudoku cell.
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
		normalise_boxes(line);

		assert!(box1.value == Some(1));
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
		normalise_boxes(row);
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
}
