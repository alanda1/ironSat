use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::{assignment::Assignment, clause::Clause};

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

    pub fn activity(&self) -> &Vec<f32> {
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

    fn forget_clause(&mut self, indexes: HashSet<usize>) {
        for index in &indexes {
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
            for removed in &indexes {
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
        for index in indexes{
            reorder.push(index);
        }
        while !reorder.is_empty(){
            let index = reorder.pop().unwrap();
            self.clauselist.remove(index);
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

    pub fn resolve_conflict_dpll(&mut self, _: usize) -> bool {
        // DPLL Conflict Resolution:

        let last_decision = &self.movelist.last().unwrap()[0];
        let var = match last_decision {
            SolverMove::Decide(val) => *val,
            _other => return false,
        };

        self.movelist.remove(self.movelist.len() - 1);
        self.add_move(SolverMove::DecideFromConflict(-1 * var, 0));
        return true;
    }
    pub fn resolve_conflict_cdcl(&mut self, clause_index: usize) -> bool {
        // println!("Begin conflict resolution");
        // dbg!(&self.movelist);
        // dbg!(self.clauselist());
        if self.movelist.len() == 0 {
            return false;
        }

        // Increase activity for all variables in conflict clause
        let conflict_clause = &self.clauselist[clause_index];
        for var in &conflict_clause.vars {
            let var_index: usize = (TryInto::<usize>::try_into((*var).abs()).unwrap()) - 1;
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
                // println!("Adding {} to variable level map", var);
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
                        // println!("Adding {} to variable clause map", *variable)
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
        // dbg!(last_decided_var);

        // Create conflict clause
        let mut conflict_list: HashSet<i32> = HashSet::new();
        for var in &conflict_clause.vars {
            conflict_list.insert(*var);
        }
        conflict_list.remove(&last_decided_var);

        let current_level = self.movelist.len() - 1;
        //Begin conflict resolution
        loop {
            let mut clause: Option<&Clause> = None;
            let mut var: Option<i32> = None;
            // dbg!(&conflict_list);
            for conflict_var in &conflict_list {
                let conflict_var_inv = -1 * conflict_var;
                if conflict_var_inv == last_decided_var {
                    continue;
                }
                // dbg!(*conflict_var);
                if *variable_level_map.get(&conflict_var_inv).unwrap() == current_level {
                    // Set Resolving clause
                    let explaining_clause =
                        &self.clauselist[*variable_clause_map.get(&conflict_var_inv).unwrap()]; //Maybe this should return if unwrap fails?
                                                                                                // dbg!(&explaining_clause.vars);
                    clause = Some(explaining_clause);
                    var = Some(*conflict_var);
                    break;
                }
            }
            if clause.is_none() {
                break;
            }

            for additional_var in &clause.unwrap().vars {
                conflict_list.insert(*additional_var);
            }
            conflict_list.remove(&var.unwrap());
            conflict_list.remove(&(var.unwrap() * -1));
        }

        // Get levels of all variables in conflict clause
        let mut new_clause_list = Vec::new();
        // dbg!(&conflict_list);
        let mut found_levels: BinaryHeap<usize> = BinaryHeap::new();
        // dbg!(current_level);
        for var in conflict_list {
            let var_inv = var * -1;
            new_clause_list.push(var);
            // dbg!(var);
            let level = *variable_level_map.get(&var_inv).unwrap();
            assert!(level != current_level || var_inv == last_decided_var);
            found_levels.push(level);
        }
        self.add_clause(Clause::from_vec(new_clause_list));
        // Get second highest level
        let first = found_levels.pop();
        let backjump_level = match found_levels.pop() {
            Some(val) => val + 1,
            None => first.unwrap(),
        };
        self.movelist.truncate(backjump_level);
        // dbg!(self.clauses());
        self.add_move(SolverMove::DecideFromConflict(
            -1 * last_decided_var,
            self.clauses() - 1,
        ));

        // Find clauses to delete
        let deletable_clauses = self.find_deletable_clauses();
        // dbg!(&deletable_clauses);
        self.forget_clause(deletable_clauses);
        // Decay all activity values
        for i in 0..self.vars() * 2 {
            self.activitylist[i] *= 0.9;
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

#[cfg(test)]
mod tests {
    use crate::{
        assignment::Assignment,
        clause::{self, AssignmentResult, Clause},
        solver_state::SolverMove,
    };

    use super::SolverState;

    #[test]
    fn example_test() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn cdcl_conflict() {
        let clause1 = Clause::from_vec(Vec::from([1]));
        let clause2 = Clause::from_vec(Vec::from([-1, 2]));
        let clause3 = Clause::from_vec(Vec::from([-3, 4]));
        let clause4 = Clause::from_vec(Vec::from([-5, -6]));
        let clause5 = Clause::from_vec(Vec::from([-1, -5, 7]));
        let clause6 = Clause::from_vec(Vec::from([-2, -5, 6, -7]));
        let clauses = Vec::from([clause1, clause2, clause3, clause4, clause5, clause6]);
        let state = SolverState {
            clauselist: clauses,
            movelist: Vec::new(),
            activitylist: vec![0.0; 7],
            vars: 7,
            original_clause_count: 6,
        };
    }
}
