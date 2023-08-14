use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Action {
    MoveRow(i8, bool),
    MoveCol(i8, bool),
    WriteCell(u8),
    SetCandidate(u8),
    RemoveCandidate(u8),
    ToggleCandidate(u8),
    ClearCell,
    CycleColor,
    // Unsure if useful when cycling between 3 choices is already fast
    // TODO SetColor(PRIMARY/SECONDARY/CLEAR)?
    ClearAllColors,
}

fn parse_action_string(value: &str) -> Option<(String, Vec<String>)> {
    // Expected format for string action should be similar to a function call
    // Meaning: action_0(), action_1(arg1), action_2(arg1, arg2), ...
    // Spacing is optional: action_2 (a1,  a2   ) is equivalent to action_2(a1,a2)
    // Case is ignored

    // Split string into 2 parts, action name and arguments
    // Check that both parts exist (opening bracket is found)
    let mut iter = value.split('(').map(String::from);
    let name = match iter.next() {
        Some(s) => s,
        None => return None,
    };

    let mut args = match iter.next() {
        Some(s) => s,
        None => return None,
    };

    // Should only have 1 opening bracket, meaning next iter item should be None
    if iter.next().is_some() {
        return None;
    }

    // Check that args ends with closing bracket
    match args.chars().last() {
        // Good char
        Some(')') => {}
        // Ends with char other than )
        Some(_) => return None,
        // Previous checks should mean there is at least 1 char in the args variable
        None => unreachable!(),
    };
    args.pop(); // Remove closing bracket from string

    // Convert args to vec
    // If action string has no arguments, args will be a vector of len 1 with an empty string
    // Replace with empty vector for correctness
    let args: Vec<String> = if args.is_empty() {
        Vec::new()
    } else {
        args.split(',')
            .map(|s| s.trim())
            .map(String::from)
            .collect()
    };
    Some((name, args))
}

impl TryFrom<String> for Action {
    type Error = ActionParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (name, args) = parse_action_string(&value).ok_or(ActionParsingError(value.clone()))?;

        // Since if-let chains are not stable yet (as of 1.71), use many match statements
        // for every argument count (allows us to access indexes without worrying about an
        // out-of-bounds error).
        // When (if) if-let chains get stabilized we can use one large match statement
        // with `if args.len() == 2 && let ...`

        // No-arg actions
        if args.is_empty() {
            return match name.to_lowercase().as_ref() {
                "cyclecolor" => Ok(Action::CycleColor),
                "clearallcolors" => Ok(Action::ClearAllColors),
                "clearcell" => Ok(Action::ClearCell),
                _ => Err(ActionParsingError(value.clone())),
            };
        }

        // 1-arg
        if args.len() == 1 {
            return match name.to_lowercase().as_ref() {
                "writecell" => {
                    if let Ok(arg) = args[0].parse() {
                        if (1..=9).contains(&arg) {
                            Some(Action::WriteCell(arg))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "setcandidate" => {
                    if let Ok(arg) = args[0].parse() {
                        if (1..=9).contains(&arg) {
                            Some(Action::SetCandidate(arg))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "removecandidate" => {
                    if let Ok(arg) = args[0].parse() {
                        if (1..=9).contains(&arg) {
                            Some(Action::RemoveCandidate(arg))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "togglecandidate" => {
                    if let Ok(arg) = args[0].parse() {
                        if (1..=9).contains(&arg) {
                            Some(Action::ToggleCandidate(arg))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
            .ok_or(ActionParsingError(value.clone()));
        }

        // 2-args
        if args.len() == 2 {
            return match name.to_lowercase().as_ref() {
                "moverow" => {
                    if let Ok(a0) = args[0].parse() {
                        Some(Action::MoveRow(a0, args[1].to_lowercase() == "true"))
                    } else {
                        None
                    }
                }
                "movecol" => {
                    if let Ok(a0) = args[0].parse() {
                        Some(Action::MoveCol(a0, args[1].to_lowercase() == "true"))
                    } else {
                        None
                    }
                }
                _ => None,
            }
            .ok_or(ActionParsingError(value.clone()));
        }

        // Invalid number of arguments in action
        Err(ActionParsingError(value))
    }
}

#[derive(Error, Debug)]
#[error("Invalid action: {0}")]
pub struct ActionParsingError(String);

/*
#[derive(Debug)]
pub struct ActionParsingError {
    action_str: String,
}

impl fmt::Display for ActionParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error trying to parse the following string into an action: {}",
            self.action_str
        )
    }
}

impl std::error::Error for ActionParsingError {}
*/
