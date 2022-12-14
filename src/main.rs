use std::cmp::{max, min};
use std::collections::HashMap;
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
    let mut moves: usize = 0;
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
                println!("Moves: {}\n{}", moves, state.to_string());
                return;
            }
            SolverMove::Conflict(index) => {
                if state.resolve_conflict(index) {
                    continue;
                } else {
                    println!("Moves: {}\nUnsat", moves);
                    return;
                }
            }
            SolverMove::DecideFromConflict(_) => panic!("Next move cannot by DecideFromConflict"),
        }
        moves += 1;
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
                Ok(val) => {
                    initial_state.set_vars(val);
                    let activity_vec = vec![0.0; val * 2];
                    // activity_vec[0] = 0.01;
                    initial_state.set_activity(activity_vec);
                }
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

    let mut clause_status = vec![false; state.clauses()]; // True indicates the clause is sat
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
                AssignmentResult::Sat() => {
                    sat_clauses += 1;
                    clause_status[clause_index] = true
                }
            },
            None => continue,
        }
    }

    if sat_clauses == state.clauses() {
        return SolverMove::Sat();
    }

    // If no move found decide
    // for var in 1..assignment.len() {
    //     if assignment[var].is_none() {
    //         return SolverMove::Decide(var as i32);
    //     }
    // }

    // let var = decide_first_unsat(&assignment, &clause_status, state.clauselist());
    // let var = decide_bohm(&assignment, &clause_status, state.clauselist());
    let var = decide_activity(&assignment, state, &clause_status, state.clauselist());
    return SolverMove::Decide(var);
}

#[allow(dead_code)]
fn decide_first_unsat(
    assignment: &Assignment,
    clause_status: &Vec<bool>,
    clauses: &Vec<Clause>,
) -> i32 {
    for index in 1..clause_status.len() {
        if clause_status[index] {
            // Skip over sat clauses
            continue;
        }
        let decide_clause = &clauses[index];
        for var in &decide_clause.vars {
            let index: usize = var.abs().try_into().unwrap();
            if assignment[index].is_none() {
                return *var;
            }
        }
    }
    panic!("No unsat clauses to decide on");
}

#[allow(dead_code)]
fn decide_bohm(assignment: &Assignment, clause_status: &Vec<bool>, clauses: &Vec<Clause>) -> i32 {
    // println!("Begin bohm with assignment {}", assignment.to_string());
    let alpha = 1;
    let beta = 2;
    // let mut length_to_clause_map: HashMap<usize, Vec<Clause>> = HashMap::new();
    // for index in 0..clauses.len() {
    //     if clause_status[index] {
    //         continue;
    //     }
    //     let clause = &clauses[index];
    //     let mut adjusted_list: Vec<i32> = Vec::new();
    //     for var in &clause.vars {
    //         let val = *var;
    //         let assignment_index: usize = val.abs().try_into().unwrap();
    //         if assignment[assignment_index].is_none(){
    //             adjusted_list.push(val);
    //         }
    //     }
    //     let adjusted_clause = Clause::from_vec(adjusted_list);
    //     let length = adjusted_clause.vars.len();
    //     if !length_to_clause_map.contains_key(&length){
    //         length_to_clause_map.insert(length, Vec::new());
    //     }
    //     length_to_clause_map.get_mut(&length).unwrap().push(adjusted_clause);
    // }

    let mut var_to_count_map_map: HashMap<i32, HashMap<usize, usize>> = HashMap::new();
    // Generate "Vectors" of counts in clauses of size n
    let mut max_clause_len = 0;
    let mut last_var = 0;
    for index in 0..clauses.len() {
        if clause_status[index] {
            continue;
        }
        let clause = &clauses[index];
        let mut adjusted_list: Vec<i32> = Vec::new();
        for var in &clause.vars {
            let val = *var;
            let assignment_index: usize = val.abs().try_into().unwrap();
            if assignment[assignment_index].is_none() {
                adjusted_list.push(val);
                last_var = val;
            }
        }
        let length = adjusted_list.len();
        max_clause_len = max(max_clause_len, length);
        for var in adjusted_list {
            if !var_to_count_map_map.contains_key(&var) {
                var_to_count_map_map.insert(var, HashMap::new());
            }

            let len_to_count_map = var_to_count_map_map.get_mut(&var).unwrap();
            if !len_to_count_map.contains_key(&length) {
                len_to_count_map.insert(length, 0);
            }
            len_to_count_map.insert(length, len_to_count_map.get(&length).unwrap() + 1);
        }
    }

    let empty: HashMap<usize, usize> = HashMap::new();
    // Find best variable to assign
    let mut best_var = last_var;
    for index in 1..=assignment.len() {
        if assignment[index].is_some() {
            continue;
        }
        let best_map = var_to_count_map_map.get(&best_var).unwrap_or(&empty);
        let best_map_inv = var_to_count_map_map.get(&(best_var * -1)).unwrap_or(&empty);
        let cur: i32 = index.try_into().unwrap();
        let cur_inv: i32 = cur * -1;
        // println!("{}",cur);
        let cur_map = var_to_count_map_map.get(&cur).unwrap_or(&empty);
        let cur_map_inv = var_to_count_map_map.get(&cur_inv).unwrap_or(&empty);
        for len in 1..=max_clause_len {
            let h_cur = match cur_map.get(&len) {
                Some(count) => *count,
                None => 0,
            };
            let h_cur_inv = match cur_map_inv.get(&len) {
                Some(count) => *count,
                None => 0,
            };
            let h_best = match best_map.get(&len) {
                Some(count) => *count,
                None => 0,
            };
            let h_best_inv = match best_map_inv.get(&len) {
                Some(count) => *count,
                None => 0,
            };

            let score_cur = alpha * max(h_cur, h_cur_inv) + beta * min(h_cur, h_cur_inv);
            let score_best = alpha * max(h_best, h_best_inv) + beta * min(h_best, h_best_inv);
            if score_best == score_cur {
                continue;
            } else if score_cur > score_best {
                best_var = cur;
                break;
            } else {
                break;
            }
        }
    }

    // Check if best_var or its inverse is best
    let mut normal_sum = 0;
    let mut inv_sum = 0;

    let best_map = var_to_count_map_map.get(&best_var).unwrap_or(&empty);
    let best_map_inv = var_to_count_map_map.get(&(best_var * -1)).unwrap_or(&empty);
    for i in 1..=max_clause_len {
        normal_sum += match best_map.get(&i) {
            Some(count) => *count,
            None => 0,
        };
        inv_sum += match best_map_inv.get(&i) {
            Some(count) => *count,
            None => 0,
        };
    }
    if normal_sum > inv_sum {
        return best_var;
    } else {
        return -1 * best_var;
    }
}

fn decide_activity(assignment: &Assignment, state: &SolverState, clause_status: &Vec<bool>, clauses: &Vec<Clause>) -> i32 {
    let mut best_activity = 0.0;
    let mut best_var: i32 = 0;
    // dbg!(state.activity());
    for i in 0..state.vars() {
        if assignment[i + 1].is_some() {
            continue;
        }
        let var_activty = state.activity()[i];
        if var_activty > best_activity {
            best_activity = var_activty;
            best_var = <usize as TryInto<i32>>::try_into(i).unwrap() + 1;
        }
    }

    for i in 0..state.vars() {
        let index = i + state.vars();
        if assignment[i + 1].is_some() {
            continue;
        }
        let var_activty = state.activity()[index];
        if var_activty > best_activity {
            best_activity = var_activty;
            best_var = (<usize as TryInto<i32>>::try_into(i).unwrap() + 1) * -1;
        }
    }
    
    if best_var == 0 {
        let output =  decide_bohm(assignment, clause_status, clauses);
        // dbg!(output);
        return output;
        
    }
    // dbg!(best_var);
    return best_var;
}
