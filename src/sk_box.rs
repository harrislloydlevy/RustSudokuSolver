use crate::constants::*;
use crossterm::style::*;
use crossterm::{cursor::*, execute};
use std::fmt;
use std::io::stdout;

// TODO: Change the from_possibles fucntions to use slices instead of vecs.

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Box {
    pub value: Option<u8>,
    pub poss: [bool; 10],
}

impl fmt::Display for Box {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            Some(x) => formatter.write_fmt(format_args!("{}", x)),
            None => formatter.write_str("."),
        }
    }
}

pub const BLANK_BOX: Box = Box {
    value: None,
    poss: [false, true, true, true, true, true, true, true, true, true],
};

const BOX_EMPTY_POSS: [bool; 10] = [
    false, false, false, false, false, false, false, false, false, false,
];

impl Box {
    /**
     * box_value
     *
     * Return a box of a particular value.
     *
     * Returns a blank box is passed 0. I know it seems dumb, but it made
     * writing up the tests easier in parts.
     */
    pub fn from_val(x: u8) -> Box {
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
    pub fn set_val(&mut self, x: u8) {
        self.value = Some(x);
        self.poss = BOX_EMPTY_POSS;
        self.poss[x as usize] = true;
    }

    /**
     * from_possibles
     *
     * Create a new box without a known value, from with a known set of possible values.
     */
    pub fn from_possibles(possibles: Vec<u8>) -> Box {
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
     * Useful when an algorithim has flagged a set of values as the only possible ones, but
     * if some of the "possible" bits have already been remove don't add them.
     *
     */
    pub fn remove_possible_bits(&mut self, possible_bits: u16) {
        let curr_poss = self.get_possibles_bits();
        self.set_possibles_bits(curr_poss & possible_bits);
    }

    /**
     * remove_possible_value
     *
     * Based on a value further restrict the box removing any values not makred as possible
     * This doesn't add any new possibilities if they are possible in the map, just removes.
     *
     */
    pub fn remove_possible_value(&mut self, value: u16) {
        self.poss[value as usize] = false;
    }

    /**
     * remove_impossible_bits
     *
     * Based on a bitmap further restrict the box removing any values makred as impossible
     * This doesn't add any new possibilities if they are possible in the map, just removes.
     *
     * Useful when an algorithim has flagged a set of values as not being possible so remove them
     */
    pub fn remove_impossible_bits(&mut self, impossible_bits: u16) {
        let curr_poss = self.get_possibles_bits();
        self.set_possibles_bits(curr_poss & !impossible_bits);
    }

    /**
     * from_possibles_bits
     *
     * Create a new box without a known value, from with a known set of possible values.
     */
    pub fn from_possibles_bits(possibles: u16) -> Box {
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
    pub fn set_possibles(&mut self, possibles: Vec<u8>) {
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
    pub fn set_possibles_bits(&mut self, possibles: u16) {
        // Can never have no options.
        assert!(possibles != 0);
        // Ensure no bits set above the 9th position by checking bitmask
        // against 01111111110;
        assert!((possibles & 0b1111111110) == possibles);

        // Check if there is only a single bit set
        if possibles == possibles & (!(possibles - 1)) {
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
                _ => assert!(false),
            }
        } else {
            // Otherwise there are multiple possible values here. Iterate over them
            let mut n = 0;
            while n <= 9 {
                self.poss[n] = (possibles >> n & 0b1) == 0b1;
                n = n + 1;
            }
        }
    }

    /**
     * get_possibles
     *
     * Get a list of the possilbe values of this box
     */
    pub fn get_possibles(&self) -> Vec<u16> {
        let mut result = Vec::new();
        for x in 1..10 {
            if self.poss[x] {
                result.push(x as u16);
            }
        }

        result
    }

    /**
     *
     * get_possible_bits
     *
     * Get a list of possible values as a bit mask.
     *
     */
    pub fn get_possibles_bits(&self) -> u16 {
        let mut result: u16 = 0;

        for x in 1..10 {
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
                let mut poss_values = [
                    false, false, false, false, false, false, false, false, false, false,
                ];
                poss_values[x as usize] = true;

                assert!(self.poss == poss_values);
            }
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

    pub fn pretty_print(&self, diff: Option<Box>) {
        // See docs for pretty print on sudoku function for full overview.
        // There are two different way to print here, one for if we have a
        // confirmed value and another if we onkly have possibles.

        // First check if we have been give a diff to check against, if
        // we have and it's actually different set a flag to show that we
        // should print the value in red to show this values is different.
        let is_diff;

        match diff {
            Some(diff_box) => is_diff = *self != diff_box,
            None => {
                is_diff = false;
            }
        }

        if is_diff {
            execute!(stdout(), SetForegroundColor(Color::Red)).ok();
        }

        match self.value {
            Some(x) => {
                execute!(stdout(), MoveRight(1), MoveDown(1)).ok();
                print!("{}", x);
                execute!(stdout(), MoveLeft(2), MoveUp(1)).ok();
            }
            None => {
                for val in 1..10 {
                    if self.poss[val] == true {
                        print!("{}", val);
                    } else {
                        print!(".");
                    }
                    if val % 3 == 0 {
                        // Every third line move back to the start and down on.
                        execute!(stdout(), MoveLeft(3), MoveDown(1)).ok();
                    }
                }
                execute!(stdout(), MoveUp(3)).ok();
            }
        }

        if is_diff {
            execute!(stdout(), ResetColor).ok();
        }
    }

    pub fn solved(&self) -> bool {
        match self.value {
            Some(_x) => {
                return true;
            }
            None => {
                return false;
            }
        }
    }
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
    let bad_value = Box {
        value: Some(11),
        poss: [
            false, false, false, false, false, false, false, false, false, false,
        ],
    };

    // This box has no value so should pass all it's test.
    bad_value.check();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    // Checks for a box with a set value, but a possibles array that doesn't match.
    fn test_bad_possibles_box() {
        let bad_value = Box {
            value: Some(4),
            poss: [
                false, true, false, false, false, true, false, false, false, false,
            ],
        };

        bad_value.check();
    }

    #[test]
    #[should_panic]
    // Checks that a box with no possibilities fails.
    fn test_has_possibles_box() {
        let bad_value = Box {
            value: None,
            poss: [
                false, false, false, false, false, false, false, false, false, false,
            ],
        };

        bad_value.check();
    }

    #[test]
    // Check that a boxes methods for updating and reading values stay consistent.
    fn test_value_set_and_read() {
        let setter = Box::from_possibles(vec![1, 4, 7]);

        assert!(setter.poss == [false, true, false, false, true, false, false, true, false, false]);

        // Now we do the same but with a bit pattern.
        let bit_pattern: u16 = ON << 1 | OFF << 2 | ON << 4 | ON << 7;
        let setter = Box::from_possibles_bits(bit_pattern);

        assert_eq!(bit_pattern, setter.get_possibles_bits());

        assert!(setter.poss == [false, true, false, false, true, false, false, true, false, false]);
    }

    #[test]
    // Check that converting back and forth from arrays of vals and bitmaps works.
    fn test_possibles_bitmaps() {
        let test_box = Box::from_possibles([1, 3, 9].to_vec());

        assert_eq!(test_box.get_possibles_bits(), ON << 1 | ON << 3 | ON << 9);
    }
}
