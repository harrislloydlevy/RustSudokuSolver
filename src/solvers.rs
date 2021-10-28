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

/**
 * only_options
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
pub fn only_options(mut boxes: Vec<&mut Box>) {
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
}
 */


/**
 * For a given count of combinations provide an array of bit patterns that represent every
 * combination of that number of values.
fn combinations_maps(factors:u16) -> Vec<u16> {
   assert!(factors > 1);
   assert!(factors < 4);

   let result = Vec::new();

   result	
}
 */

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
fn combo(pool: &[u16], k:u16) -> Vec<u16> {
  println!("{:?} for {}", pool, k);
  let mut result = Vec::new();

  if k > (pool.len() as u16) {
    println!("{} < {} - Terminating", k, pool.len());
    return result;
  }

  for i in 0..pool.len() {
    println!("{} of {} for {}", i, pool.len(), k);
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
      let subresults = combo(&pool[i+1..],k-1);

      for element in subresults {
		// Now add on all those combinations with the starting number
        // we already have.
        result.push(base_bit_pattern | element);
      }
	}
  }

  result
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
		normalise_boxes(line);

		assert!(box1.value == Some(1));
	}

    #[test]
	fn test_simple_combo() {
        let result = combo(&[1, 2, 3, 4], 2);
		// Combos we want to see are:
        // 1,2 / 1,3 / 1,4 / 2,3 / 2,4 / 3,4
		for element in result.iter() {
			println!("{:#018b}, ", element);
		}
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
		for element in result.iter() {
			println!("{:#018b}, ", element);
		}
		// Not checking everyone. There are 126 possible combinations of 4 objects from
        // a pool of 9 so justt check that.
		assert_eq!(result.len(), 126);
	}
}
