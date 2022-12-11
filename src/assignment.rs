use std::ops::Index;

use crate::solver_state::SolverMove;

pub struct Assignment {
    assignments: Vec<Option<bool>>,
}

impl Assignment {
    pub fn from_movelist(list: &Vec<SolverMove>, vars: usize) -> Assignment {
        let mut initial_assignments: Vec<Option<bool>> = vec![None; vars];
        for assignment in list {
            let variable: i32 = match assignment {
                SolverMove::Propagate {
                    variable,
                    clause: _,
                } => *variable,
                SolverMove::Decide(variable) => *variable,
                SolverMove::DecideFromConflict(variable) => *variable,
                SolverMove::Sat() => {
                    panic!("Attempted to generate assignment from completed movelist")
                }
                SolverMove::Conflict(_) => {
                    panic!("Attempted to generate assignment from movelist with conflict")
                }
            };
            let index: usize = variable.abs() as usize;
            initial_assignments[index - 1] = Some(variable > 0); //Variable '1' maps to assignments[0]
        }

        return Assignment {
            assignments: initial_assignments,
        };
    }

    pub fn len(&self) -> usize {
        return self.assignments.len();
    }
}

impl ToString for Assignment {
    fn to_string(&self) -> String {
        let mut buf = "".to_owned();
        for i in 1..=self.assignments.len() {
            match self[i] {
                Some(val) => {
                    if val {
                        buf = buf + &i.to_string() + " ";
                    } else {
                        buf = buf + "-" + &i.to_string() + " ";
                    }
                }
                None => continue,
            }
        }

        return buf;
    }
}

impl Index<usize> for Assignment {
    type Output = Option<bool>;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.assignments[index - 1];
    }
}
