use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use crate::game_state::{Cell, GameState};

const SIZE: usize = 600;
const PADDING: usize = 3;
const FONT_SIZE: usize = 50;
const CANDIDATE_SIZE: usize = 15;

#[wasm_bindgen]
pub struct GridRenderer {
    ctx: CanvasRenderingContext2d,
    cell_size: usize,
}

impl Default for GridRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GridRenderer {
    fn clear_canvas(&self) {
        self.ctx.clear_rect(0., 0., SIZE as f64, SIZE as f64);
    }

    /// Draws the grid lines
    fn draw_grid(&self) {
        // Size might not be a multiple of 9, make sure lines don't overflow
        let line_end = (self.cell_size * 9 + PADDING + 1) as f64;

        self.ctx.set_stroke_style(&"rgba(0,0,0,1.0)".into());

        // Draw horizontal lines
        for (i, y) in (PADDING..SIZE).step_by(self.cell_size).enumerate() {
            self.ctx.begin_path();
            // Draw heavier lines for box outlines
            if i % 3 == 0 {
                self.ctx.set_line_width(3.0);
            } else {
                self.ctx.set_line_width(1.0);
            }
            let y = y as f64;
            self.ctx.move_to((PADDING - 1) as f64, y);
            self.ctx.line_to(line_end, y);
            self.ctx.stroke();
        }

        // Vertical lines
        for (i, x) in (PADDING..SIZE).step_by(self.cell_size).enumerate() {
            self.ctx.begin_path();
            if i % 3 == 0 {
                self.ctx.set_line_width(3.0);
            } else {
                self.ctx.set_line_width(1.0);
            }

            let x = x as f64;
            self.ctx.move_to(x, (PADDING - 1) as f64);
            self.ctx.line_to(x, line_end);
            self.ctx.stroke();
        }
    }

    fn draw_cells(&self, game_state: &GameState) {
        let grid = game_state.grid();
        let focused_cell = game_state.focused_cell();

        for (row, row_vec) in grid.iter().enumerate() {
            for (col, cell) in row_vec.iter().enumerate() {
                // Change focused cell's border color
                // TODO change bg color of cells with same digit as focused cell
                if row as u8 == focused_cell.0 && col as u8 == focused_cell.1 {
                    let (top_y, top_x) = self.get_cell_pos(row, col);

                    // Draw highlighted border
                    self.ctx.set_stroke_style(&"rgba(230,60,255,1)".into());
                    self.ctx.set_line_width(3.);
                    self.ctx.stroke_rect(
                        top_x as f64,
                        top_y as f64,
                        self.cell_size as f64,
                        self.cell_size as f64,
                    );
                }

                if cell.digit.is_some() {
                    self.write_cell_digit(row, col, cell);
                } else {
                    // Render candidates
                    self.write_cell_candidates(row, col, &cell.candidates);
                }
            }
        }
    }

    fn write_cell_digit(&self, row: usize, col: usize, cell: &Cell) {
        assert!(cell.digit.is_some());
        let digit = cell.digit.unwrap();
        assert!((1..=9).contains(&digit));

        let (row_pos, col_pos) = self.get_cell_pos(row, col);
        // Returned values point to top-left corner of cell, but we want
        // y to be bottom of text and x to be center of text.
        let x_pos = col_pos + self.cell_size / 2;
        let y_pos = row_pos + 4 + self.cell_size / 2; // Y pos needs a small offset for some reason

        // Set digit color (differenciate givens and user inputs)
        if cell.is_given {
            // Black
            self.ctx.set_fill_style(&"rgba(0,0,0,1)".into());
        } else {
            // Purple
            self.ctx.set_fill_style(&"rgba(230,60,255,1)".into());
        }

        self.ctx.set_font(&format!("{}px consolas", FONT_SIZE));
        self.ctx.set_text_align("center");
        self.ctx.set_text_baseline("middle");
        self.ctx
            .fill_text(&digit.to_string(), x_pos as f64, y_pos as f64)
            .unwrap();
    }

    fn write_cell_candidates(&self, row: usize, col: usize, candidates: &[bool; 9]) {
        let (row_pos, col_pos) = self.get_cell_pos(row, col);
        for (n, has_candidate) in candidates.iter().enumerate() {
            if !has_candidate {
                // Cell doesn't have this candidate, skip
                continue;
            }
            // Calculate candidate digit offset in cell
            // Weird math but trust that it makes sense (fancy way of aligning along thirds with
            // some padding on both sides of the axis)
            const CANDIDATE_PADDING: usize = 2;
            let offset_size = (self.cell_size - 2 * CANDIDATE_PADDING) / 6;
            let x_offset = CANDIDATE_PADDING + (2 * (n % 3) + 1) * offset_size;
            let y_offset = CANDIDATE_PADDING + 3 + (2 * (n / 3) + 1) * offset_size;
            let x_pos = (col_pos + x_offset) as f64;
            let y_pos = (row_pos + y_offset) as f64;

            self.ctx.set_fill_style(&"rgba(20,20,20,1)".into()); // Dark gray
            self.ctx.set_font(&format!("{}px consolas", CANDIDATE_SIZE));
            self.ctx.set_text_align("center");
            self.ctx.set_text_baseline("middle");
            self.ctx
                .fill_text(&(n + 1).to_string(), x_pos, y_pos)
                .unwrap();
        }
    }

    /// Obtains the top-left coordinate of a specific cell
    fn get_cell_pos(&self, row: usize, col: usize) -> (usize, usize) {
        // Logic: Padding + n * 1px for lines separating cells + n * cell_size
        let row_pos = PADDING + (row * self.cell_size);
        let col_pos = PADDING + (col * self.cell_size);
        (row_pos, col_pos)
    }
}

// Methods exported to JS
#[wasm_bindgen]
impl GridRenderer {
    pub fn new() -> Self {
        // Obtain 2d context for canvas
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        // Calculate cell_size
        let cell_size = (SIZE - 2 * PADDING) / 9;

        Self { ctx, cell_size }
    }

    /// Renders the grid to the canvas
    pub fn render(&self, game_state: &GameState) {
        self.clear_canvas();
        self.draw_grid();
        self.draw_cells(game_state);
    }
}
