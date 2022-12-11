use crate::assignment::Assignment;

pub enum AssignmentResult {
    Propagate(i32),
    Conflict(),
    Sat(),
}

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