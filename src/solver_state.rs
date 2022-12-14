use crate::{assignment::Assignment, clause::Clause};

#[derive(Clone)]
pub enum SolverMove {
    Propagate { variable: i32, clause: usize },
    Decide(i32),
    DecideFromConflict(i32),
    Sat(),
    Conflict(usize),
}

pub struct SolverState {
    clauselist: Vec<Clause>,
    movelist: Vec<Vec<SolverMove>>,
    activitylist: Vec<f32>,
    vars: usize,
    clauses: usize,
}

impl SolverState {
    pub fn new() -> SolverState {
        SolverState {
            clauselist: Vec::new(),
            movelist: Vec::new(),
            vars: 0,
            clauses: 0,
            activitylist: Vec::new(),
        }
    }

    pub fn activity(&self) -> &Vec<f32>{
        return &self.activitylist;
    }

    pub fn add_clause(&mut self, clause: Clause) {
        self.clauselist.push(clause);
    }

    pub fn add_move(&mut self, item: SolverMove) {
        match self.movelist.last_mut() {
            Some(last_level) => last_level.push(item),
            None => {
                self.movelist.push(Vec::new());
                self.movelist[0].push(item)
            }
        }
    }

    pub fn add_decision_lv(&mut self) {
        self.movelist.push(Vec::new());
    }

    pub fn clauselist(&self) -> &Vec<Clause> {
        return &self.clauselist;
    }

    pub fn clauses(&self) -> usize {
        return self.clauses;
    }
    pub fn get_movelist(&self) -> Vec<SolverMove> {
        let mut moves: Vec<SolverMove> = Vec::new();
        for level in &self.movelist {
            for item in level {
                moves.push(item.clone());
            }
        }
        return moves;
    }

    pub fn resolve_conflict(&mut self, clause_index: usize) -> bool {
        if self.movelist.len() == 0 {
            return false;
        }

        // Increase activity for all variables in conflict clause
        let conflict_clause = &self.clauselist[clause_index];
        for var in &conflict_clause.vars {
            let var_index: usize = (TryInto::<usize>::try_into((*var).abs()).unwrap()) -1;
            let index: usize = if *var > 0 {
                var_index
            } else {
                var_index + self.vars
            };
            // print!("{} ", var);
            let old = self.activitylist[index];
            self.activitylist[index] = old + 1.0;
        }
        // println!("");

        let last_decision = &self.movelist.last().unwrap()[0];
        let var = match last_decision {
            SolverMove::Decide(val) => *val,
            _other => return false,
        };

        self.movelist.remove(self.movelist.len() - 1);
        self.add_move(SolverMove::DecideFromConflict(-1 * var));

        // Decay all activity values
        for i in 0..self.vars()*2 {
          self.activitylist[i] *= 0.9;  
        }

        return true;
    }

    pub fn set_activity(&mut self, activity: Vec<f32>) {
        self.activitylist = activity;
    }
    pub fn set_clauses(&mut self, clauses: usize) {
        self.clauses = clauses;
    }

    pub fn set_vars(&mut self, vars: usize) {
        self.vars = vars;
    }
    pub fn vars(&self) -> usize {
        return self.vars;
    }
}

impl ToString for SolverState {
    fn to_string(&self) -> String {
        let mut buf = "".to_owned();
        // buf = buf + "Clauses:\n";
        // for clause in &self.clauselist {
        //     for var in &clause.vars {
        //         buf = buf + &var.to_string() + " ";
        //     }
        //     buf = buf + "\n";
        // }
        buf = buf + "\nAssignment:\n";

        let assignment = Assignment::from_movelist(&self.get_movelist(), self.vars());
        buf = buf + &assignment.to_string();
        return buf.to_string();
    }
}
