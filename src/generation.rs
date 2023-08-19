use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};

/// Creates a fully completed Sudoku grid
pub fn generate_random_filled_grid() -> Vec<Vec<u8>> {
    let mut grid = vec![vec![0; 9]; 9];
    // Fill boxes 1, 5 and 9 randomly since they never interact with eachother
    let mut rng = rand::thread_rng();
    let offsets = [0, 3, 6];

    for offset in offsets {
        let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        digits.shuffle(&mut rng);
        for (i, digit) in digits.iter().enumerate() {
            let row = (i / 3) + offset;
            let col = (i % 3) + offset;
            grid[row][col] = *digit;
        }
    }

    match fill_grid(grid, &mut rng) {
        Some(grid) => grid,
        None => panic!("Unable to fill grid"),
    }
}

/// Recursively fills cells in the grid until everything is filled
fn fill_grid(grid: Vec<Vec<u8>>, rng: &mut ThreadRng) -> Option<Vec<Vec<u8>>> {
    // Find first empty cell
    let (row_idx, col_idx) = match get_first_empty_index(&grid) {
        Some((r, c)) => (r, c),
        None => return Some(grid), // No empty cell means grid is fully filled
    };

    let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    digits.shuffle(rng);

    for digit in digits {
        if is_safe_placement(&grid, row_idx, col_idx, digit) {
            //let grid_copy: Vec<Vec<u8>> = grid.iter().cloned().collect();
            let mut grid_copy: Vec<Vec<u8>> = grid.clone();
            grid_copy[row_idx][col_idx] = digit;

            if let Some(g) = fill_grid(grid_copy, rng) {
                return Some(g); // Bubbling up filled grid
            }
            // Didn't return = no solution possible with this digit, try next digit
        }
    }

    // No solution possible with any digits, backtrack
    None
}

/// Counts the number of solution a grid has.
/// Works similarily to the `fill_grid` function, but bubbles up the number
/// of solutions instead of the filled grid
fn solution_count(grid: Vec<Vec<u8>>) -> usize {
    // Find first empty cell
    let (row_idx, col_idx) = match get_first_empty_index(&grid) {
        Some((r, c)) => (r, c),
        None => return 1, // No empty -> grid is filled (1 solution)
    };

    let mut solutions = 0;
    for digit in [1, 2, 3, 4, 5, 6, 7, 8, 9] {
        if is_safe_placement(&grid, row_idx, col_idx, digit) {
            let mut grid_copy = grid.clone();
            grid_copy[row_idx][col_idx] = digit;
            solutions += solution_count(grid_copy);
        }
    }
    solutions
}

/// Masks a filled grid until `given_count` cells remain
fn mask_grid(grid: Vec<Vec<u8>>, given_count: usize) -> Vec<Vec<u8>> {
    // TODO figure out randomness (seeded optimally)
    // This function could reach a state where no removal actions would result in a unique
    // situation, in which case the function would get stuck in a loop. Add a safeguard if it
    // occurs often (doubt it should be common)
    let mut rng = rand::thread_rng();
    assert!(given_count >= 17); // Need at least 17 clues to have unique solution
    let mut mask_count = 9 * 9 - given_count;
    let mut removed = 0;

    // First 20 removals done in quads
    let mut masked_grid = grid.clone();
    while mask_count >= 4 && removed < 20 {
        // TODO Cells 1-4 could have some overlap with each other. Maybe validate there's no
        //  overlap if worthwhile?
        let (c1_r, c1_c) = get_random_unmasked_cell(&grid, &mut rng);
        let (c2_r, c2_c) = get_random_unmasked_cell(&grid, &mut rng);
        let (c3_r, c3_c) = get_jittery_mirrored_cell(&grid, c1_r, c1_c, &mut rng);
        let (c4_r, c4_c) = get_jittery_mirrored_cell(&grid, c2_r, c2_c, &mut rng);

        // Mask the cells
        masked_grid[c1_r][c1_c] = 0;
        masked_grid[c2_r][c2_c] = 0;
        masked_grid[c3_r][c3_c] = 0;
        masked_grid[c4_r][c4_c] = 0;

        if solution_count(masked_grid.clone()) == 1 {
            mask_count -= 4;
            removed += 4;
        } else {
            // Multiple solution with removals, restore cells and try other quad
            masked_grid[c1_r][c1_c] = grid[c1_r][c1_c];
            masked_grid[c2_r][c2_c] = grid[c2_r][c2_c];
            masked_grid[c3_r][c3_c] = grid[c3_r][c3_c];
            masked_grid[c4_r][c4_c] = grid[c4_r][c4_c];
        }
    }

    // Remove cells in mirrored pairs
    while mask_count >= 2 && removed < 30 {
        let (c1_r, c1_c) = get_random_unmasked_cell(&grid, &mut rng);
        let (c2_r, c2_c) = get_jittery_mirrored_cell(&grid, c1_r, c1_c, &mut rng);

        masked_grid[c1_r][c1_c] = 0;
        masked_grid[c2_r][c2_c] = 0;

        if solution_count(masked_grid.clone()) == 1 {
            mask_count -= 2;
            removed += 2;
        } else {
            // Puzzle has 1+ solution, restore cells and choose new ones
            masked_grid[c1_r][c1_c] = grid[c1_r][c1_c];
            masked_grid[c2_r][c2_c] = grid[c2_r][c2_c];
        }
    }

    // Remove remaining cells individually
    while mask_count >= 1 {
        let (cell_r, cell_c) = get_random_unmasked_cell(&grid, &mut rng);
        masked_grid[cell_r][cell_c] = 0;

        if solution_count(masked_grid.clone()) == 1 {
            mask_count -= 1;
        } else {
            masked_grid[cell_r][cell_c] = grid[cell_r][cell_c];
        }
    }

    masked_grid
}

fn get_first_empty_index(grid: &[Vec<u8>]) -> Option<(usize, usize)> {
    let flat_index = match grid
        .iter()
        .flat_map(|r| r.iter())
        .enumerate()
        .find(|(_, &val)| val == 0)
    {
        Some((idx, _)) => idx,
        None => return None,
    };

    // Convert flat index to 2d indexes
    let row_idx = flat_index / 9;
    let col_idx = flat_index % 9;
    Some((row_idx, col_idx))
}

fn get_random_unmasked_cell(grid: &[Vec<u8>], rng: &mut ThreadRng) -> (usize, usize) {
    // Function assumes there is at least 1 non-zero cell
    loop {
        let row = rng.gen_range(0..9);
        let col = rng.gen_range(0..9);
        if grid[row][col] != 0 {
            return (row, col);
        }
    }
}

fn get_jittery_mirrored_cell(
    grid: &[Vec<u8>],
    row: usize,
    col: usize,
    rng: &mut ThreadRng,
) -> (usize, usize) {
    let mirror_r = 9 - row as isize - 1;
    let mirror_c = 9 - col as isize - 1;
    loop {
        // Give small offsets to mirrored position
        // Under & overflows loop around
        let new_r = mirror_r + rng.gen_range(-3..=3);
        let new_r = ((new_r + 9) % 9) as usize;
        let new_c = mirror_c + rng.gen_range(-3..=3);
        let new_c = ((new_c + 9) % 9) as usize;

        if grid[new_r][new_c] != 0 {
            return (new_r, new_c);
        }
    }
}

/// Checks if grid is still valid after placing new digit in a specified cell
fn is_safe_placement(grid: &[Vec<u8>], row: usize, col: usize, val: u8) -> bool {
    // Check if row still valid
    let mut seen = [false; 9];
    seen[val as usize - 1] = true;
    for elem in &grid[row] {
        if *elem == 0 {
            continue;
        }
        let idx = *elem as usize - 1;
        if seen[idx] {
            // Digit already seen before
            return false;
        }
        seen[idx] = true;
    }

    // Check for col
    let mut seen = [false; 9];
    seen[val as usize - 1] = true;
    for row in grid.iter() {
        let elem = row[col];
        if elem == 0 {
            continue;
        }
        let idx = elem as usize - 1;
        if seen[idx] {
            return false;
        }
        seen[idx] = true;
    }

    // Check for box
    let box_row = row / 3;
    let box_col = col / 3;
    let mut seen = [false; 9];
    seen[val as usize - 1] = true;
    for r in 0..3 {
        for c in 0..3 {
            let row_idx = 3 * box_row + r;
            let col_idx = 3 * box_col + c;
            let elem = grid[row_idx][col_idx];
            if elem == 0 {
                continue;
            }
            let idx = elem as usize - 1;
            if seen[idx] {
                return false;
            }
            seen[idx] = true;
        }
    }
    true
}

// Unsure why clippy detects as dead code when it is the main function
#[allow(dead_code)]
pub fn main() {
    let complete = generate_random_filled_grid();
    print_grid(&complete);
    let masked = mask_grid(complete, 25);
    println!();
    print_grid(&masked);
}

/// Pretty-prints the grid
fn print_grid(grid: &[Vec<u8>]) {
    for (i, row) in grid.iter().enumerate() {
        for (j, elem) in row.iter().enumerate() {
            print!("{} ", elem);
            if j % 3 == 2 {
                print!(" ") // Extra space
            }
        }
        println!();
        if i % 3 == 2 {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_solution_grid() {
        // Known sudoku grid to have a unique solution
        let grid: Vec<Vec<u8>> = [
            vec![0, 1, 0, 0, 2, 0, 3, 0, 4],
            vec![0, 0, 2, 0, 0, 5, 6, 1, 0],
            vec![7, 0, 0, 0, 0, 3, 0, 8, 0],
            vec![5, 0, 6, 0, 4, 0, 0, 0, 1],
            vec![0, 0, 1, 0, 0, 0, 2, 0, 0],
            vec![9, 0, 0, 0, 7, 0, 4, 0, 5],
            vec![0, 4, 0, 6, 0, 0, 0, 0, 9],
            vec![0, 6, 7, 2, 0, 0, 5, 0, 0],
            vec![2, 0, 8, 0, 1, 0, 0, 3, 0],
        ]
        .to_vec();

        assert_eq!(solution_count(grid), 1);
    }

    #[test]
    fn test_many_solutions_grid() {
        // Proved to have 5 solutions by 3rd party validators
        let grid: Vec<Vec<u8>> = [
            vec![0, 0, 0, 0, 2, 0, 3, 0, 4],
            vec![0, 0, 2, 0, 0, 5, 6, 1, 0],
            vec![7, 0, 0, 0, 0, 3, 0, 8, 0],
            vec![5, 0, 6, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 1, 0, 0, 0, 2, 0, 0],
            vec![9, 0, 0, 0, 7, 0, 4, 0, 5],
            vec![0, 4, 0, 0, 0, 0, 0, 0, 9],
            vec![0, 6, 7, 0, 0, 0, 5, 0, 0],
            vec![2, 0, 8, 0, 1, 0, 0, 0, 0],
        ]
        .to_vec();

        assert_eq!(solution_count(grid), 5);
    }
}
