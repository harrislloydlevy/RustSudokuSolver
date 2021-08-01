
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
struct Box {
    value: u8,
    poss: [bool; 10]
}

struct Cell {
    boxes: [Box; 9]
}

struct Sudoku {
    cells: [Cell; 9]
}

// Consts to easily get the index of a given positions in a 3x3 array that's stored
// as an array.
const TOP_LFT:u8 = 0;
const TOP_MID:u8 = 1;
const TOP_RHT:u8 = 2;
const MID_LFT:u8 = 3;
const MID_MID:u8 = 4;
const MID_RHT:u8 = 5;
const BOT_LFT:u8 = 6;
const BOT_MID:u8 = 7;
const BOT_RHT:u8 = 8;

fn read_sudoku() {
    // We expect to read a stream of numbers set out in the same
    // way a sudo would be printed on page, with "|" and "-" marks
    // used to break up the cells and the boxes in each cell just seperated by
    // spaces. Like this:
    
    // To read that stream into our more strutued 3- level tree we iterate
    // over:
    // 1. First over each of the 3 rows of cells in the sudoku.
    let cur_cel_row = 0;

    // 2. Then over each of 3 rows of boxes insides those cells
    let cur_box_row = 0;

    // 3. The over the 3 cells that cross the row of numbers
    let cur_cel_col = 0;

    // 4. Then we iterate over the boxes within that particular cell
    let cur_box_col = 0;

    // These iterations then update the current cell, and the curernt box to
    // read the next value into.
    let cur_box = 0;
    let cur_cel = 0;

    for cur_cel_row in 0..3 {
        println!("-------------");
        for cur_box_row in 0..3 {
            print!("|");
            for cur_cel_col in 0..3 {
                for cur_box_col in 0..3 {
                    //println!("C{}{}B{}{}|", cur_cel_col, cur_cel_row, cur_box_row, cur_box_col);
                    print!("{}", cur_box_row*3+cur_box_col);
                }
                print!("|");
            }
            println!();
        }
    }
    println!("-------------");

}


fn main() {
    println!("Hello, world!");

   read_sudoku();
}
