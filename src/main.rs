// TODO: 'reduce' fucntuon to merge two sudokus of possibles and solving as 'map' function
//       as part of making multi-threaded
// TODO  Clean up all the unused fucntions that are used in tests but not in main code to be
//       used from main code

mod constants;
mod sk_box;
mod sk_cell;
mod solvers;
mod sudoku;
mod sudoku_builder;
use crate::sudoku::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut filename: &String = &"".to_string();
    let mut file_type: FileType = FileType::Simple;
    let mut sudokus: Vec<sudoku::Sudoku> = Vec::new();
    //Simple,
    //Pretty,
    //Multi,
    //Possibles,

    if args.len() == 2 {
        // One arg means just a filename to to solve, type already
        // defaulted to 'Simple'
        filename = &args[1];
    } else if args.len() == 3 {
        // First argument to be a switch in the form "-X" where X
        // denotes the type of files to read.
        assert!(args[1].len() == 2);
        assert!(args[1].starts_with('-'));
        file_type = match args[1].chars().nth(1).unwrap() {
            's' => FileType::Simple,
            'p' => FileType::Pretty,
            'm' => FileType::Multi,
            'o' => FileType::Possibles,
            _ => FileType::Simple,
        };

        filename = &args[2];
    }

    // Parse the input file provided into a sudoku depending on the type
    if matches!(file_type, FileType::Simple) {
        sudokus.push(sudoku::Sudoku::from_ss(filename.to_string()).unwrap());
    } else if matches!(file_type, FileType::Pretty) {
        sudokus.push(sudoku::Sudoku::from_pretty(filename.to_string()).unwrap());
    } else if matches!(file_type, FileType::Possibles) {
        sudokus.push(Sudoku::from_possibles(filename.to_string()));
    } else if matches!(file_type, FileType::Multi) {
        sudokus = Sudoku::from_txt(filename.to_string());
    }

    // Now just solve all the sudokus in the vector. Will only be one for most cases.
    //
    for mut sudoku in sudokus {
        sudoku.pretty_print(None, Some("Solving".to_string()));

        sudoku.solve();

        if sudoku.solved() {
            sudoku.print_ss();
        } else {
            sudoku.pretty_print(None, Some("Incomplete Solve".to_string()));
        }
    }
    return;
}
