use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use assignment::Assignment;
use clause::{AssignmentResult, Clause};
use solver_state::SolverState;

use crate::solver_state::SolverMove;

mod assignment;
mod clause;
mod solver_state;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args[1];
    println!("Looking for file: {file}");
    let initial_config = parse_input(file);

    if let Err(e) = initial_config {
        println!("Error parsing: {e}");
        return;
    }

    let mut state = initial_config.unwrap();
    // println!("{}", state.to_string());

    loop {
        let next_move = move_from_state(&state);
        match next_move {
            SolverMove::Propagate { variable, clause } => state.add_move(SolverMove::Propagate {
                variable: variable,
                clause: clause,
            }),
            SolverMove::Decide(var) => {
                state.add_decision_lv();
                state.add_move(SolverMove::Decide(var))
            }
            SolverMove::Sat() => {
                println!("{}", state.to_string());
                return;
            }
            SolverMove::Conflict(_) => if state.resolve_conflict() {
                continue
            } else {
                println!("Unsat");
                return;
            },
            SolverMove::DecideFromConflict(_) => panic!("Next move cannot by DecideFromConflict"),
        }
    }
    // dbg!(args);
}

fn parse_input(path: &str) -> Result<SolverState, Box<dyn Error>> {
    let file = File::open(path).expect("Couldn't open file");

    let reader = BufReader::new(file);

    let mut initial_state = SolverState::new();
    let header_initialized = false;
    for line in reader.lines() {
        let line = line?;
        if line.chars().nth(0).unwrap() == 'c' {
            continue;
        }

        if line.chars().nth(0).unwrap() == '%' {
            break;
        }

        let splits: Vec<&str> = line.split_whitespace().collect();

        if splits[0] == "p" {
            if header_initialized {
                return Err("Header initialized multiple times".into());
            }

            if splits.len() != 4 {
                return Err("Bad header format".into());
            }

            if splits[1] != "cnf" {
                return Err("Header does not indicate cnf".into());
            }

            let parsed_variables = splits[2].parse::<usize>();
            match parsed_variables {
                Ok(val) => initial_state.set_vars(val),
                Err(_) => return Err(format!("Variable count must be a number").into()),
            }

            let parsed_clauses = splits[3].parse::<usize>();
            match parsed_clauses {
                Ok(val) => initial_state.set_clauses(val),
                Err(_) => return Err(format!("Clause count must be a number").into()),
            }
            continue;
        }
        let mut clause: Vec<i32> = Vec::new();
        for num in splits {
            let parsed_num = num.parse::<i32>();

            match parsed_num {
                Ok(val) => {
                    if val != 0 {
                        clause.push(val)
                    }
                }
                Err(_) => return Err(format!("{num} is not a number").into()),
            };
        }
        initial_state.add_clause(Clause::from_vec(clause));
        // dbg!(clause);
    }

    Ok(initial_state)
}

fn move_from_state(state: &SolverState) -> SolverMove {
    let assignment = Assignment::from_movelist(&state.get_movelist(), state.vars());
    let mut sat_clauses = 0;
    // Loop through clauses and check for possible propagates or conflicts
    for clause_index in 0..state.clauselist().len() {
        let clause = &state.clauselist()[clause_index];
        let clause_result = clause.check_assignment(&assignment);

        match clause_result {
            Some(status) => match status {
                AssignmentResult::Propagate(var) => {
                    return SolverMove::Propagate {
                        variable: var,
                        clause: clause_index,
                    }
                }
                AssignmentResult::Conflict() => return SolverMove::Conflict(clause_index),
                AssignmentResult::Sat() => sat_clauses += 1,
            },
            None => continue,
        }
    }

    if sat_clauses == state.clauses() {
        return SolverMove::Sat();
    }

    // If no move found decide
    for var in 1..assignment.len() {
        if assignment[var].is_none() {
            return SolverMove::Decide(var as i32);
        }
    }
    panic!("No possible decisions found")
}
