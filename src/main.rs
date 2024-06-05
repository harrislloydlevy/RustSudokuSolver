// TODO: Move each solving function out to it's own library and make each generic signature
// TODO: Add new solving function where if 'x' boxes all can _only_ be the same 'x' values then
//       any other boxes in their row/cell can't be those values
// TODO: 'reduce' fucntuon to merge two sudokus of possibles and solving as 'map' function
// TODO: Hide more of the internal structure of the structs inside the modules
// TODO: Easier way to get the values/posibles of a specific cell/box than directly looking up
//       the internal structures.

mod constants;
mod sk_box;
mod sk_cell;
mod solvers;
mod sudoku;
mod sudoku_builder;

fn main() {
    let mut sudoku = sudoku::Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();
    sudoku.print_ss();

    while !sudoku.solved() {
        sudoku.pretty_print(None);

        let orig = sudoku;

        // Try naive solving
        println!("Apply Naive Solve");
        let pre_single_pos = sudoku;
        solvers::single_position(&mut sudoku);
        sudoku.pretty_print(Some(pre_single_pos));

        // Then try some more advanced/expensive methods
        println!("Apply Candidate Line");
        let pre_candidate = sudoku;
        solvers::candidate_line(&mut sudoku);
        sudoku.pretty_print(Some(pre_candidate));

        // If we made no progress at all over the whole last round - then we don't have the
        // abiliyt to solve this sudoku.
        if orig == sudoku {
            println!("Could not solve sudoku.");
            return;
        }
    }
    println!("Solved sudoku!");
    return;
}
