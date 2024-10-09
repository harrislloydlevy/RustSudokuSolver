// TODO: 'reduce' fucntuon to merge two sudokus of possibles and solving as 'map' function
//       as part of making multi-threaded
// TODO  Load sudoku from a file that shows possibles as well as actual values to make debugging
//       easier
// TODO  Clean up all the unused fucntions that are used in tests but not in main code to be
//       used from main code

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
        sudoku.pretty_print(None, None);

        let orig = sudoku;

        // Try naive solving
        println!("Apply Single Position");
        let pre_single_pos = sudoku;
        solvers::single_position(&mut sudoku);
        sudoku.pretty_print(Some(pre_single_pos), None);

        println!("Apply Naked Set");
        let pre_ns = sudoku;
        solvers::naked_set(&mut sudoku);
        sudoku.pretty_print(Some(pre_ns), None);

        // Then try some more advanced/expensive methods
        println!("Apply Candidate Line");
        let pre_candidate = sudoku;
        solvers::candidate_line(&mut sudoku);
        sudoku.pretty_print(Some(pre_candidate), None);

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
