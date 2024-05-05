// TODO: Move each solving function out to it's own library and make each generic signature
// TODO: Add new solving function where if 'x' boxes all can _only_ be the same 'x' values then
//       any other boxes in their row/cell can't be those values
// TODO: Conitinual solving over the sudoku trying new solving fucntions until done/stuck
// TODO: 'reduce' fucntuon to merge two sudokus of possibles and solving as 'map' function
// TODO: Hide more of the internal structure of the structs inside the modules
// TODO: Easier way to get the values/posibles of a specific cell/box than directly looking up
//       the internal structures.

mod sk_box;
mod sk_cell;
mod solvers;
mod constants;
mod sudoku_builder;
mod sudoku;

fn main() {
    let mut sudoku = sudoku::Sudoku::from_ss("test/easy_solve.ss".to_string()).unwrap();
    solvers::naive(&mut sudoku);
    sudoku.print_ss();
}
