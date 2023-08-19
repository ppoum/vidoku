use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::{console, KeyboardEvent};

use crate::{
    actions::Action,
    generation,
    key::Key,
    keybinds::{Keybind, KeybindManager},
};

#[derive(Clone, Copy)]
pub struct Cell {
    pub digit: Option<u8>,
    pub candidates: [bool; 9],
    pub is_given: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

impl Cell {
    pub fn new() -> Self {
        Cell {
            digit: None,
            candidates: [false; 9],
            is_given: false,
        }
    }

    pub fn clear_candidates(&mut self) {
        self.candidates = [false; 9];
    }
}

#[wasm_bindgen]
pub struct GameState {
    kb_manager: KeybindManager,
    last_key: Rc<RefCell<Option<Keybind>>>,
    grid: Vec<Vec<Cell>>,
    solution: Vec<Vec<u8>>,
    focused_row: u8,
    focused_col: u8,
    highlighted_digit: Option<u8>,
    // Refactor game options into their own struct
    show_errors: bool,
}

impl GameState {
    pub fn grid(&self) -> &Vec<Vec<Cell>> {
        &self.grid
    }

    pub fn focused_cell_coord(&self) -> (u8, u8) {
        (self.focused_row, self.focused_col)
    }

    pub fn highlighted_digit(&self) -> Option<u8> {
        self.highlighted_digit
    }

    pub fn show_errors(&self) -> bool {
        self.show_errors
    }

    pub fn expected_value(&self, row: usize, col: usize) -> u8 {
        self.solution[row][col]
    }

    /// Reads the last key pressed without consuming it.
    pub fn peek_last_key(&self) -> Option<Keybind> {
        // RefCell can only be mutably borrowed by the keyboard event
        // callback when #consume_last_key is called, which should drop the borrow quickly.
        // Loop until we can get an immutable borrow
        loop {
            // Only return if borrow succeeded
            if let Ok(x) = self.last_key.try_borrow() {
                return *x;
            }
        }
    }

    /// Consumes and returns the last key pressed. Further calls to this function
    /// without a new key press happening will result in a return value of `None`.
    pub fn consume_last_key(&mut self) -> Option<Keybind> {
        loop {
            if let Ok(mut x) = self.last_key.try_borrow_mut() {
                let val = *x;
                *x = None;
                return val;
            }
        }
    }

    pub fn get_focused_cell(&self) -> &Cell {
        &self.grid[self.focused_row as usize][self.focused_col as usize]
    }

    fn get_mut_focused_cell(&mut self) -> &mut Cell {
        &mut self.grid[self.focused_row as usize][self.focused_col as usize]
    }
}

// Methods exported to JS
#[wasm_bindgen]
impl GameState {
    /// Creates a new `GameState` object and registers a `keydown` event listener
    pub fn with_keybind_manager(kb_manager: KeybindManager) -> Self {
        let last_key_mtx: Rc<RefCell<Option<Keybind>>> = Rc::new(RefCell::new(None));

        let kb_callback;
        {
            let last_key_mtx = last_key_mtx.clone();
            let kb_manager = kb_manager.clone();

            kb_callback = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                loop {
                    if e.ctrl_key() {
                        // Block event if a keybind is registered with same key
                        let keybind = Keybind {
                            key: e.key().try_into().unwrap_or(Key::Zero),
                            modifier: Some(Key::Control),
                        };
                        if kb_manager.get_action(&keybind).is_some() {
                            e.prevent_default();
                        }
                    }

                    // Loop until we can obtain a mutable borrow
                    if let Ok(mut x) = last_key_mtx.try_borrow_mut() {
                        // Shift+digit results in #key returning the associated special character
                        // instead of the digit (meaning S-1..S-9) binds don't work. Add
                        // edge case handling
                        let mut key: Option<Key> = None;
                        if e.shift_key()
                            && (e.code().starts_with("Digit") || e.code().starts_with("Numpad"))
                        {
                            // Edge-case use code (Digitn or Numpadn) to generate Key object
                            let key_digit = e.code().chars().last().unwrap();
                            if key_digit.is_ascii_digit() {
                                key = Some(key_digit.to_string().try_into().unwrap());
                            }
                        }

                        if key.is_none() {
                            // Edge-case didn't apply, do normal logic with e.key
                            // Map unknown keys to 0 (probably should warn users in console)
                            key = Some(e.key().try_into().unwrap_or(Key::Zero));
                        }
                        let key = key.unwrap();

                        // If key event is for pressing down on a modifier key, ignore
                        // (as in: ignore when user presses down on Shift itself, as shift cannot
                        // be binded by itself)
                        if key.is_modifier() {
                            break;
                        }

                        let modifier = if e.shift_key() {
                            Some(Key::Shift)
                        } else if e.ctrl_key() {
                            Some(Key::Control)
                        } else if e.alt_key() {
                            Some(Key::Alt)
                        } else if e.meta_key() {
                            Some(Key::Meta)
                        } else {
                            None
                        };

                        *x = Some(Keybind { key, modifier });
                        console::debug_1(&format!("{:?} (k:{},c:{})", x, e.key(), e.code()).into());
                        break;
                    }
                }
            }) as Box<dyn FnMut(_)>);
        }

        // Register callback on "keydown" event on canvas element
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();

        canvas
            .add_event_listener_with_callback("keydown", kb_callback.as_ref().unchecked_ref())
            .unwrap();

        // Callback closure needs to outlive this method call.
        // "Forget" the object so that rust doesn't destroy it
        kb_callback.forget();

        Self {
            kb_manager,
            last_key: last_key_mtx,
            grid: vec![vec![Cell::new(); 9]; 9],
            solution: vec![vec![0; 9]; 9],
            focused_row: 0,
            focused_col: 0,
            highlighted_digit: None,
            show_errors: true,
        }
    }

    /// Updates the game state based on the user's inputs
    pub fn update(&mut self) {
        if let Some(keybind) = self.consume_last_key() {
            if let Some(action) = self.kb_manager.get_action(&keybind) {
                match action {
                    Action::MoveRow(n, safe) => {
                        // If safe, only move if not out of bounds
                        // If not safe, move and cap to border of grid if overflow
                        let new_pos = self.focused_row as i8 + n;
                        if *safe {
                            if (0..9).contains(&new_pos) {
                                self.focused_row = new_pos as u8;
                            }
                        } else {
                            self.focused_row = new_pos.clamp(0, 8) as u8;
                        }
                    }
                    Action::MoveCol(n, safe) => {
                        // Safe same as MoveRow
                        let new_pos = self.focused_col as i8 + n;
                        if *safe {
                            if (0..9).contains(&new_pos) {
                                self.focused_col = new_pos as u8;
                            }
                        } else {
                            self.focused_col = new_pos.clamp(0, 8) as u8;
                        }
                    }
                    Action::WriteCell(n) => {
                        if self.get_focused_cell().is_given {
                            return;
                        }
                        self.get_mut_focused_cell().digit = Some(*n);
                        self.get_mut_focused_cell().clear_candidates();
                    }
                    Action::ClearCell => {
                        if self.get_focused_cell().is_given {
                            return;
                        }
                        self.get_mut_focused_cell().digit = None
                    }
                    Action::SetCandidate(n) => {
                        if self.get_focused_cell().is_given
                            || self.get_focused_cell().digit.is_some()
                        {
                            return;
                        }

                        let n = *n as usize - 1;
                        self.get_mut_focused_cell().candidates[n] = true
                    }
                    Action::RemoveCandidate(n) => {
                        if self.get_focused_cell().is_given
                            || self.get_focused_cell().digit.is_some()
                        {
                            return;
                        }
                        let n = *n as usize - 1;
                        self.get_mut_focused_cell().candidates[n] = false
                    }
                    Action::ToggleCandidate(n) => {
                        if self.get_focused_cell().is_given
                            || self.get_focused_cell().digit.is_some()
                        {
                            return;
                        }
                        let n = *n as usize - 1;
                        let curr_val = self.get_mut_focused_cell().candidates[n];
                        self.get_mut_focused_cell().candidates[n] = !curr_val;
                    }
                    Action::ClearCandidates => {
                        self.get_mut_focused_cell().clear_candidates();
                    }
                    Action::HighlightCurrentDigit => {
                        self.highlighted_digit = self.get_focused_cell().digit;
                    }
                    Action::HighlightDigit(n) => {
                        self.highlighted_digit = Some(*n);
                    }
                    Action::ClearHighlight => {
                        self.highlighted_digit = None;
                    }
                    _ => todo!("Remaining actions: {:?}", action),
                }
            }
        }
    }
    pub fn generate_grid(&mut self, seed: String, given_count: usize) {
        let (solution, grid) = generation::generate_grid(seed, given_count);

        // Map grid u8 to Cell
        let grid = grid
            .into_iter()
            .map(|r| {
                r.into_iter()
                    .map(|n| match n {
                        // 0 means masked cell
                        0 => Cell {
                            digit: None,
                            candidates: [false; 9],
                            is_given: false,
                        },
                        // Other digit means given cell
                        n => Cell {
                            digit: Some(n),
                            candidates: [false; 9],
                            is_given: true,
                        },
                    })
                    .collect()
            })
            .collect();
        self.grid = grid;

        self.solution = solution;
    }
}
