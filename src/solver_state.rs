use crate::{assignment::Assignment, clause::Clause};
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Clone, Debug)]
pub enum SolverMove {
    Propagate { variable: i32, clause: usize },
    Decide(i32),
    DecideFromConflict(i32, usize),
    Sat(),
    Conflict(usize),
}

pub struct SolverState {
    clauselist: Vec<Clause>,
    movelist: Vec<Vec<SolverMove>>,
    activitylist: Vec<f32>,
    vars: usize,
    pub original_clause_count: usize,
}

impl SolverState {
    pub fn new() -> SolverState {
        SolverState {
            clauselist: Vec::new(),
            movelist: Vec::new(),
            vars: 0,
            activitylist: Vec::new(),
            original_clause_count: 0,
        }
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

    fn bump_activity(&mut self, var: i32) {
        let var_index: usize = (TryInto::<usize>::try_into((var).abs()).unwrap()) - 1;
        let index: usize = if var > 0 {
            var_index
        } else {
            var_index + self.vars
        };
        let old = self.activitylist[index];
        self.activitylist[index] = old + 1.0;
    }

    pub fn get_activity(&self, var: i32) -> f32 {
        let var_index: usize = (TryInto::<usize>::try_into((var).abs()).unwrap()) - 1;
        let index: usize = if var > 0 {
            var_index
        } else {
            var_index + self.vars
        };
        return self.activitylist[index];
    }

    pub fn clauselist(&self) -> &Vec<Clause> {
        return &self.clauselist;
    }

    pub fn clauses(&self) -> usize {
        return self.clauselist.len();
    }

    fn find_deletable_clauses(&self) -> HashSet<usize> {
        let mut all_set: HashSet<usize> = HashSet::new();
        for i in self.original_clause_count..self.clauses() {
            all_set.insert(i);
        }

        for level in &self.movelist {
            for solver_move in level {
                match solver_move {
                    SolverMove::Propagate {
                        variable: _,
                        clause,
                    } => all_set.remove(clause),
                    SolverMove::DecideFromConflict(_, clause) => all_set.remove(clause),
                    _ => continue,
                };
            }
        }

        return all_set;
    }

    fn forget_clause(&mut self, indexes: &HashSet<usize>) {
        for index in indexes {
            assert!(*index > (self.original_clause_count - 1));
        }

        #[cfg(debug_assertions)]
        {
            for level in &self.movelist {
                for solver_move in level {
                    match solver_move {
                        SolverMove::Propagate {
                            variable: _,
                            clause,
                        } => assert!(!indexes.contains(clause)),
                        SolverMove::DecideFromConflict(_, clause) => {
                            assert!(!indexes.contains(clause))
                        }
                        _ => continue,
                    }
                }
            }
        }

        // Calculate new clause index values for existing moves
        let mut index_remapping: HashMap<usize, usize> = HashMap::new();
        for i in 0..self.clauses() {
            let mut num_less = 0;
            for removed in indexes {
                if *removed < i {
                    num_less += 1;
                }
            }
            let new_val = i - num_less;
            index_remapping.insert(i, new_val);
        }

        // Update clause index values for existing moves
        for level in 0..self.movelist.len() {
            for i in 0..self.movelist[level].len() {
                let clause = &self.movelist[level][i];
                match clause {
                    SolverMove::Propagate { variable, clause } => {
                        self.movelist[level][i] = SolverMove::Propagate {
                            variable: *variable,
                            clause: index_remapping[clause],
                        }
                    }
                    SolverMove::DecideFromConflict(var, clause) => {
                        self.movelist[level][i] =
                            SolverMove::DecideFromConflict(*var, index_remapping[clause])
                    }
                    _ => continue,
                }
            }
        }

        let mut reorder = BinaryHeap::new();
        for index in indexes {
            reorder.push(index);
        }
        while !reorder.is_empty() {
            let index = reorder.pop().unwrap();
            self.clauselist.remove(*index);
        }
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

    #[allow(dead_code)]
    pub fn resolve_conflict_dpll(&mut self, clause_index: usize) -> bool {
        // DPLL Conflict Resolution:

        // Increase activity for all variables in conflict clause
        let conflict_clause = self.clauselist[clause_index].vars.clone();
        for var in &conflict_clause {
            self.bump_activity(*var);
        }

        let last_decision = &self.movelist.last().unwrap()[0];
        let var = match last_decision {
            SolverMove::Decide(val) => *val,
            _other => return false,
        };

        self.movelist.remove(self.movelist.len() - 1);
        self.add_move(SolverMove::DecideFromConflict(-1 * var, 0));

        for i in 0..self.vars() * 2 {
            self.activitylist[i] *= 0.95;
        }

        return true;
    }

    #[allow(dead_code)]
    pub fn resolve_conflict_cdcl(&mut self, clause_index: usize) -> bool {
        if self.movelist.len() == 0 {
            // Nowhere to backjump to
            return false;
        }

        // Build structure for variable -> decision level
        let mut variable_level_map: HashMap<i32, usize> = HashMap::new();
        for level in 0..self.movelist.len() {
            for solver_move in &self.movelist[level] {
                let var = match solver_move {
                    SolverMove::Propagate {
                        variable,
                        clause: _,
                    } => *variable,
                    SolverMove::Decide(val) => *val,
                    SolverMove::DecideFromConflict(val, _) => *val,
                    SolverMove::Sat() => panic!("Sat found when resolving conflict"),
                    SolverMove::Conflict(_) => panic!("Conflict in movelist"),
                };
                variable_level_map.insert(var, level);
            }
        }

        // Build structure for variable -> which clause explains it
        let mut variable_clause_map: HashMap<i32, usize> = HashMap::new();
        for level in 0..self.movelist.len() {
            for solver_move in &self.movelist[level] {
                match solver_move {
                    SolverMove::Propagate { variable, clause } => {
                        variable_clause_map.insert(*variable, *clause);
                    }
                    SolverMove::DecideFromConflict(variable, clause) => {
                        variable_clause_map.insert(*variable, *clause);
                    }
                    SolverMove::Sat() => panic!("Sat found when resolving conflict"),
                    SolverMove::Conflict(_) => panic!("Conflict in movelist"),
                    _other => continue,
                };
            }
        }

        let last_decision = &self.movelist.last().unwrap()[0];
        let last_decided_var = match last_decision {
            SolverMove::Decide(val) => *val,
            _other => return false,
        };

        // Create conflict clause
        let mut conflict_list: HashSet<i32> = HashSet::new();
        let conflict_clause = &self.clauselist[clause_index];
        for var in &conflict_clause.vars {
            conflict_list.insert(*var);
        }
        conflict_list.remove(&last_decided_var);

        let current_level = self.movelist.len() - 1;
        //Begin conflict resolution

        let mut active_vars = HashSet::new();
        loop {
            let mut clause: Option<&Clause> = None;
            let mut var: Option<i32> = None;

            for conflict_var in &conflict_list {
                let conflict_var_inv = -1 * conflict_var;
                // Allow the last decided variable to stay
                if conflict_var_inv == last_decided_var {
                    continue;
                }

                // Check if the variable is in the current decision level
                if *variable_level_map.get(&conflict_var_inv).unwrap() == current_level {
                    // Set Resolving clause
                    let explaining_clause =
                        &self.clauselist[*variable_clause_map.get(&conflict_var_inv).unwrap()]; //Maybe this should return if unwrap fails?
                    clause = Some(explaining_clause);
                    var = Some(*conflict_var);
                    break;
                }
            }

            // If no variable other than last decided literal is in the current decision level stop
            if clause.is_none() {
                break;
            }

            // Resolve conflict and explaining clauses
            for additional_var in &clause.unwrap().vars {
                conflict_list.insert(*additional_var);
                active_vars.insert(*additional_var);
            }
            conflict_list.remove(&var.unwrap());
            conflict_list.remove(&(var.unwrap() * -1));
        }



        // Get levels of all variables in conflict clause
        let mut new_clause_list = Vec::new();
        let mut found_levels: BinaryHeap<usize> = BinaryHeap::new();

        // Calculate levels, create clause, and update activity for learned clause
        for var in conflict_list {
            let var_inv = var * -1;
            new_clause_list.push(var);
            self.bump_activity(var);
            let level = *variable_level_map.get(&var_inv).unwrap();
            assert!(level != current_level || var_inv == last_decided_var);
            found_levels.push(level);
        }

        // Learn the conflict clause
        self.add_clause(Clause::from_vec(new_clause_list));

        // Get second highest level
        let first = found_levels.pop();
        let backjump_level = match found_levels.pop() {
            Some(val) => val + 1,
            None => first.unwrap(),
        };

        // Backjump
        self.movelist.truncate(backjump_level);
        self.add_move(SolverMove::DecideFromConflict(
            -1 * last_decided_var,
            self.clauses() - 1,
        ));

        // Find clauses to delete
        let check_forget_cutoff = 0.05;
        let check_forget_rand = rand::random::<f32>();
        let forget = check_forget_cutoff > check_forget_rand;
        if forget {
            let deletable_clauses = self.find_deletable_clauses();
            let mut filtered_deleteable_clauses = HashSet::new();
            let cutoff: f64 =
                0.0001 * (deletable_clauses.len() as f64) / (self.original_clause_count as f64);
            for index in deletable_clauses {
                let x = rand::random::<f64>();
                if x < cutoff {
                    filtered_deleteable_clauses.insert(index);
                }
            }
            self.forget_clause(&filtered_deleteable_clauses);
        }

        // Decay all activity values
        for i in 0..self.vars() * 2 {
            self.activitylist[i] *= 0.50;
        }

        return true;
    }

    pub fn set_activity(&mut self, activity: Vec<f32>) {
        self.activitylist = activity;
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
        buf = buf + "\nAssignment:\n";

        let assignment = Assignment::from_movelist(&self.get_movelist(), self.vars());
        buf = buf + &assignment.to_string();
        return buf.to_string();
    }
}
