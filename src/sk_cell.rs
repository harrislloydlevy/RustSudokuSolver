use std::fmt;
use std::io::stdout;
use crate::sk_box::*;
use crate::solvers;
use crate::constants::*;
// use boxy::{Char, Weight};

use crossterm::{
    execute,
    cursor::*
};

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Cell {
    pub boxes: [Box; 9]
}

// Instantiating these structures from nothing can be time consuming so we pre-define a series of
// blank versions of each struct for convenience later here.

pub const BLANK_CELL:Cell = Cell {
    boxes: [BLANK_BOX, BLANK_BOX, BLANK_BOX,
            BLANK_BOX, BLANK_BOX, BLANK_BOX,
            BLANK_BOX, BLANK_BOX, BLANK_BOX]
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
	// Run "normalization" which is the simplest form of solving any 9 element sudoku
	// over this particular cell.
    pub fn solve(&mut self) {
		// We can't just call normalise_boxes direclty as that fucntion expect an array
		// of references to boxes instead of an array of boxes. So we have to pass
		// in an array of refs instead.

		{ 
		  let mut vec = Vec::new();
		  for x in self.boxes.iter_mut() {
			vec.push(x);
		  }

		  solvers::single_position_candidate(vec);
        }

        {
          let mut vec = Vec::new();
		  for x in self.boxes.iter_mut() {
			vec.push(x);
		  }
          solvers::naked_set(vec);
        }
	}

	/**
     *
     * bitmap_possibles
     *
     * Get a 10 element array where each element is a bitmap of which boxes
     * in the cell as possibles for the value of the arrays index. I.e. index
     * 4 holds a bitmap of which elements in the box could possibly hold the
	 * value 4.
     *
     * This gives us an array we can easily check how often a given value
     * is possible and in whch boxes which is useful for many solving techniques.
     * 
     * For example in an cell where the values 1 and 3 where only possible in boxes
     * at index 4 and 5 and the array would have the value b1010 and indexes 4 and 5.
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
            let bit_pattern = 1 << idx;

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
	pub fn rm_poss_from_row(&mut self, value:u16, row:usize) {
		let start_idx = row*3;
		self.boxes[start_idx   ].remove_possible_value(value);
		self.boxes[start_idx + 1].remove_possible_value(value);
		self.boxes[start_idx + 2].remove_possible_value(value);
	}

	/**
	 * rm_poss_from_col
	 * 
	 * Convenience function to remove any possibilities of a given value in a given
	 * row in a cell.
	 */
	 pub fn rm_poss_from_col(&mut self, value:u16, col:usize) {
		self.boxes[col    ].remove_possible_value(value);
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
	pub fn pretty_print(&self) {
		// Start by assuming we are at the top-left corner for cusor position anyway
		// then draw out the interior lines of the cell.

		// For each row of boxes in the cell start by drawing the outline
		for row_idx in 0 ..3 {
			// Draw the horizontal lines down the centre first, 3 times
			for _i in 0 .. 3 {
			  print!("   │   │");
				execute!(stdout(), MoveLeft(8),MoveDown(1)).ok();
			}

			// Aftger the first and second add an inbetween line
			if row_idx == 0 || row_idx == 1 {
			  print!("───┼───┼───");
				execute!(stdout(), MoveLeft(11),MoveDown(1)).ok();
			}

		}
		// Now we are positioned below the box of our cell, inline with the first charachter
		// so we move back up to the top
		execute!(stdout(), MoveUp(11)).ok();

		// Now we print each of the 9 boxes in the cell to fill them in
		for row_idx in 0 .. 3 {
			for cell_idx in 0 .. 3 {
				self.boxes[(row_idx * 3)+cell_idx].pretty_print();
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
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	// Check that we can solve a box when there's only one value left.
	fn test_last_value_box() {
		let mut test_cell:Cell = Cell {
			boxes:[ BLANK_BOX,        Box::from_val(2), Box::from_val(3),
                    Box::from_val(4), Box::from_val(5), Box::from_val(6),		
                    Box::from_val(7), Box::from_val(8), Box::from_val(9)]
		};
 
		test_cell.solve();

		assert!(test_cell.boxes[TOP_LFT].value == Some(1));
	}
}
