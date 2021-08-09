use std::fs;
use std::io;
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
const BLANK_BOX:Box = Box {
    value: None,
    poss: [false, true, true, true, true, true, true, true, true, true ]
};

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

/*
 * cell_set
 *
 * Return a cell of a particular value.
 */
fn box_value(x:u8) -> Box {
    let mut result = BLANK_BOX;
    result.value = Some(x);
    result.poss = [false, false, false, false, false, false, false, false, false, false];
    result.poss[x as usize] = true;

    return result;
}

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
    // | 1 |012|012|
    // |345|345|345|
    // |678| 7 |678|
    // -------------
    // | 12|912|912|
    // |345|345|345|
    // |678|678|678|
    // -------------
    // | 12| 12| 12|
    // |345|345|345|
    // |678|678|678|
   
    // Attempt to open the file
    let file = fs::File::open(filename);
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error)
    };

    let mut reader = std::io::BufReader::new(file);

    // Instantiatie sudoko as blank
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
            println!("{} / {}: {}", cur_cel_row, cur_box_row, line);

            // Read charachters off from the RIGHT of the string using the pop
            // function. So first read off the \n and tehn continue right to
            // left.
            assert_eq!(line.pop(), Some('\n'));
            assert_eq!(line.pop(), Some('\r'));

            // From 3 to 0 because we're going from right to left!
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

                    println!("read {}/{}: {}", cur_cel_col, cur_box_col, char);

                    // 
                    if char == '.' {
                        value = BLANK_BOX;
                    } else {
                        let digit = char.to_digit(10).expect("Expected number or '.'");
                        assert!(digit >= 1 && digit <= 9, "Expected a number between 1 and 9");
                        value = box_value(digit as u8);
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
    }
}
