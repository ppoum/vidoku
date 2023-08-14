use rand::{rngs::ThreadRng, seq::SliceRandom};

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
    let index = match grid
        .iter()
        .flat_map(|r| r.iter())
        .enumerate()
        .find(|(_, &val)| val == 0)
    {
        Some((idx, _)) => idx,
        None => return Some(grid), // No empty cell means grid is filled
    };

    // Convert flat index to 2d index
    let row_idx = index / 9;
    let col_idx = index % 9;

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

/// Check if grid is still valid after placing new digit in a specified cell
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
    print_grid(&generate_random_filled_grid());
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
