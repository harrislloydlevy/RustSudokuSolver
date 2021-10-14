use std::fmt;
use crate::sk_box::*;
use crate::solvers;
use crate::constants::*;

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
    pub fn normalise(&mut self) {
		// We can't just call normalise_boxes direclty as that fucntion expect an array
		// of references to boxes instead of an array of boxes. So we have to pass
		// in an array of refs instead.

		let mut vec = Vec::new();
		for x in self.boxes.iter_mut() {
			vec.push(x);
		}

		solvers::normalise_boxes(vec);
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
 
		test_cell.normalise();

		assert!(test_cell.boxes[TOP_LFT].value == Some(1));
	}
}