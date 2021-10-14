use crate::constants::*;
use crate::sk_box::Box;

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
pub fn normalise_boxes(mut boxes: Vec<&mut Box>) {
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

#[cfg(test)]
mod tests {
    // Inherit everything from up a level so we can run functions from there.
    use super::*;
	use crate::sk_box::BLANK_BOX;

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
}