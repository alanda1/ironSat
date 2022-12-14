use crate::assignment::Assignment;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum AssignmentResult {
    Propagate(i32),
    Conflict(),
    Sat(),
}

#[derive(Debug)]
pub struct Clause {
    pub vars: Vec<i32>,
}

impl Clause {
    pub fn from_vec(varlist: Vec<i32>) -> Clause {
        Clause { vars: varlist }
    }

    pub fn check_assignment(&self, assignment: &Assignment) -> Option<AssignmentResult> {
        let mut last_available: Option<i32> = None;
        for var in &self.vars {
            let index: usize = var.abs() as usize;
            let clause_val = *var > 0;
            let var_assignment = assignment[index];
            match var_assignment {
                Some(value) => {
                    if value == clause_val  {
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

#[cfg(test)]
mod tests {
    use crate::{solver_state::SolverMove, assignment::Assignment, clause::AssignmentResult};

    #[test]
    fn example_test() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn basic_conflict(){
        let test_vec = vec![1,3,5];
        let clause = super::Clause::from_vec(test_vec);
        let test_movelist = vec![SolverMove::Decide(-1), SolverMove::Decide(-3), SolverMove::Decide(-5)];
        let assignment = Assignment::from_movelist(&test_movelist, 5);
        println!("{}", assignment.to_string());
        let result = clause.check_assignment(&assignment);
        assert_eq!(result.unwrap(), AssignmentResult::Conflict());

    }

    #[test]
    fn basic_propagate(){
        let test_vec = vec![1,3,5];
        let clause = super::Clause::from_vec(test_vec);
        let test_movelist = vec![SolverMove::Decide(-1), SolverMove::Decide(-3)];
        let assignment = Assignment::from_movelist(&test_movelist, 5);
        println!("{}", assignment.to_string());
        let result = clause.check_assignment(&assignment);
        assert_eq!(result.unwrap(), AssignmentResult::Propagate(5));

    }

    #[test]
    fn basic_sat(){
        let test_vec = vec![1,3,5];
        let clause = super::Clause::from_vec(test_vec);
        let test_movelist = vec![SolverMove::Decide(-1), SolverMove::Decide(5)];
        let assignment = Assignment::from_movelist(&test_movelist, 5);
        println!("{}", assignment.to_string());
        let result = clause.check_assignment(&assignment);
        assert_eq!(result.unwrap(), AssignmentResult::Sat());

    }

    #[test]
    fn basic_none(){
        let test_vec = vec![1,3,5];
        let clause = super::Clause::from_vec(test_vec);
        let test_movelist = vec![SolverMove::Decide(-1)];
        let assignment = Assignment::from_movelist(&test_movelist, 5);
        println!("{}", assignment.to_string());
        let result = clause.check_assignment(&assignment);
        assert_eq!(result, None);

    }

    #[test]
    fn negative_none(){
        let test_vec = vec![-1,-3,-5];
        let clause = super::Clause::from_vec(test_vec);
        let test_movelist = vec![SolverMove::Decide(1)];
        let assignment = Assignment::from_movelist(&test_movelist, 5);
        println!("{}", assignment.to_string());
        let result = clause.check_assignment(&assignment);
        assert_eq!(result, None);

    }
    
}