use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Index;

enum SolverMove {
    Propagate { variable: i32, clause: usize },
    Decide(i32),
    Sat(),
    Conflict(usize),
}

enum AssignmentResult {
    Propagate(i32),
    Conflict(),
    Sat(),
}
struct SolverState {
    clauselist: Vec<Clause>,
    movelist: Vec<Vec<SolverMove>>,
}

struct Clause {
    vars: Vec<i32>,
}

struct Assignment {
    assignments: Vec<Option<bool>>,
}

impl SolverState {
    fn new() -> SolverState {
        SolverState {
            clauselist: Vec::new(),
            movelist: Vec::new(),
        }
    }

    fn add_clause(&mut self, clause: Clause) {
        self.clauselist.push(clause);
    }
}

impl ToString for SolverState {
    fn to_string(&self) -> String {
        let mut buf = "".to_owned();
        buf = buf + "Clauses:\n";
        for clause in &self.clauselist {
            for var in &clause.vars {
                buf = buf + &var.to_string() + " ";
            }
            buf = buf + "\n";
        }
        buf = buf + "Done!";
        return buf.to_string();
    }
}

impl Clause {
    fn from_vec(varlist: Vec<i32>) -> Clause {
        Clause { vars: varlist }
    }

    fn check_assignment(&self, assignment: &Assignment) -> Option<AssignmentResult> {
        let mut last_available: Option<i32> = None;
        for var in &self.vars {
            let index: usize = var.abs() as usize;

            let var_assignment = assignment[index];
            match var_assignment {
                Some(value) => {
                    if value {
                        return Some(AssignmentResult::Sat());
                    }
                }
                None => {
                    if last_available.is_some() {
                        // There are two available variables that could be assigned so no propagate
                        return None;
                    } else {
                        last_available = Some(*var)
                    }
                }
            }
        }

        match last_available {
            Some(var) => Some(AssignmentResult::Propagate(var)),
            None => Some(AssignmentResult::Conflict()),
        }
    }
}

impl Assignment {
    fn from_movelist(list: &Vec<Vec<SolverMove>>, vars: usize) -> Assignment {
        let mut initial_assignments: Vec<Option<bool>> = vec![None; vars+1]; //0 will always be empty
        for level in list {
            for assignment in level {
                let variable: i32 = match assignment {
                    SolverMove::Propagate {
                        variable,
                        clause: _,
                    } => *variable,
                    SolverMove::Decide(variable) => *variable,
                    SolverMove::Sat() => panic!("Attempted to generate assignment from completed movelist"),
                    SolverMove::Conflict(_) => panic!("Attempted to generate assignment from movelist with conflict"),
                };
                let index: usize = variable.abs() as usize;
                initial_assignments[index] = Some(variable > 0);
            }
        }

        return Assignment {
            assignments: initial_assignments,
        };
    }

    fn len(&self) -> usize {
        return self.assignments.len();
    }
}

impl Index<usize> for Assignment {
    type Output = Option<bool>;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.assignments[index];
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args[1];
    println!("Looking for file: {file}");
    let initial_config = parse_input(file);

    if let Err(e) = initial_config {
        println!("Error parsing: {e}");
        return;
    }

    let state = initial_config.unwrap();
    println!("{}", state.to_string());

    loop {
        let next_move = move_from_state(&state);
        match next_move {
            SolverMove::Propagate { variable, clause } => todo!(),
            SolverMove::Decide(_) => todo!(),
            SolverMove::Sat() => todo!(),
            SolverMove::Conflict(_) => todo!(),
        }
    }
    // dbg!(args);


}

fn parse_input(path: &str) -> Result<SolverState, Box<dyn Error>> {
    let file = File::open(path).expect("Couldn't open file");

    let reader = BufReader::new(file);

    let mut initial_state = SolverState::new();
    let header_initialized = false;
    let mut clauses: usize = 0;
    let mut variables: usize = 0;
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
            // println!("{}",line);
            // println!("{}",splits.len());
            // for item in &splits {
            //     dbg!(item);
            // }
            if splits.len() != 4 {
                return Err("Bad header format".into());
            }

            if splits[1] != "cnf" {
                return Err("Header does not indicate cnf".into());
            }

            let parsed_variables = splits[2].parse::<usize>();
            match parsed_variables {
                Ok(val) => variables = val,
                Err(_) => return Err(format!("Variable count must be a number").into()),
            }

            let parsed_clauses = splits[3].parse::<usize>();
            match parsed_clauses {
                Ok(val) => clauses = val,
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

    println!("{} {}", variables, clauses);
    Ok(initial_state)
}

fn move_from_state(state: &SolverState) -> SolverMove{
    let len = state.clauselist.len();
    let assignment = Assignment::from_movelist(&state.movelist, len);
    let mut sat_clauses = 0;
    // Loop through clauses and check for possible propagates or conflicts
    for clause_index in 0..state.clauselist.len()  {
        let clause = &state.clauselist[clause_index];
        let clause_result = clause.check_assignment(&assignment);
        
        match clause_result {
            Some(status) => match status {
                AssignmentResult::Propagate(var) => return SolverMove::Propagate { variable: var, clause: clause_index },
                AssignmentResult::Conflict() => return SolverMove::Conflict(clause_index),
                AssignmentResult::Sat() => sat_clauses+=1,
            },
            None => continue,
        }
    }

    if sat_clauses == len{
        return SolverMove::Sat();
    }

    // If no move found decide
    for var in 1..assignment.len(){
        if assignment[var].is_none(){
            return SolverMove::Decide(var as i32);
        }
    }
    panic!("No possible decisions found")
}