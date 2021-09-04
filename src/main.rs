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
#[derive(PartialEq, Debug)]
struct Box {
    value: Option<u8>,
    poss: [bool; 10]
}

const ON:u16 = 1;
const OFF:u16 = 1;

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
     */
    fn from_value(x:u8) -> Box {
        let mut result = BLANK_BOX;
        result.value = Some(x);
        result.poss = BOX_EMPTY_POSS;
        result.poss[x as usize] = true;
    
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
     * from_possibles
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

#[derive(PartialEq, Debug)]
struct Cell {
    boxes: [Box; 9]
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

// This function creates a sudoku from a file. I don't knwo enough rust
// yet to have it return a more generic error so just using io::Error
//
// File Format taken from Simple Sudoku
fn read_simple_sudoku(filename:String) -> Result<Sudoku, io::Error> {
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
                        value = Box::from_value(digit as u8);
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

fn write_simple_sudoku(sudoku:Sudoku) {
    println!("-----------");
    for cur_cel_row in 0..3 {
        for cur_box_row in 0..3 {
            println!("{}{}{}|{}{}{}|{}{}{}",
                     sudoku.cells[cur_cel_row*3  ].boxes[cur_box_row*3  ],
                     sudoku.cells[cur_cel_row*3  ].boxes[cur_box_row*3+1],
                     sudoku.cells[cur_cel_row*3  ].boxes[cur_box_row*3+2],
                     sudoku.cells[cur_cel_row*3+1].boxes[cur_box_row*3  ],
                     sudoku.cells[cur_cel_row*3+1].boxes[cur_box_row*3+1],
                     sudoku.cells[cur_cel_row*3+1].boxes[cur_box_row*3+2],
                     sudoku.cells[cur_cel_row*3+2].boxes[cur_box_row*3  ],
                     sudoku.cells[cur_cel_row*3+2].boxes[cur_box_row*3+1],
                     sudoku.cells[cur_cel_row*3+2].boxes[cur_box_row*3+2]);
         }
         println!("-----------");
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
        let result = read_simple_sudoku("test/blank.ss".to_string()).unwrap();
        assert_eq!(result, BLANK_SUDOKU);
    }

    #[test]
    fn test_sparse_read() {
        // Test that reading and writing a sudoku works.
        // We test this by reading a sudoku from a test file, then writing out it back and and
        // ensuring the two values are the same.
        let result = read_simple_sudoku("test/sparse.ss".to_string()).unwrap();
        assert_eq!(result.cells[TOP_LFT].boxes[TOP_MID].value, Some(1));
        assert_eq!(result.cells[TOP_LFT].boxes[MID_MID].value, Some(2));
        assert_eq!(result.cells[TOP_LFT].boxes[BOT_MID].value, Some(3));

        assert_eq!(result.cells[MID_MID].boxes[TOP_MID].value, Some(1));
        assert_eq!(result.cells[MID_MID].boxes[MID_MID].value, Some(2));
        assert_eq!(result.cells[MID_MID].boxes[BOT_MID].value, Some(3));

        assert_eq!(result.cells[BOT_RHT].boxes[TOP_MID].value, Some(1));
        assert_eq!(result.cells[BOT_RHT].boxes[MID_MID].value, Some(2));
        assert_eq!(result.cells[BOT_RHT].boxes[BOT_MID].value, Some(3));

        write_simple_sudoku(result);
    }

	#[test]
	fn test_ok_value_box() {

		// Ensure box with a single value passes
		let ok_value_box = Box::from_value(2);
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
		let bit_pattern:u16 = ON << 1 | ON << 4 | ON <<7;
		let setter = Box::from_possibles_bits(bit_pattern);

		assert!(setter.poss ==
			[false, true, false, false, true, false, false, true, false, false]);

	}
}
