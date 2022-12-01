use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = &args[1];
    println!("Looking for file: {file}");
    let config = parse_input(file);

    if let Err(e) = config {
        println!("Error parsing: {e}");
        return
    }
    dbg!(args);
}

fn parse_input(path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(path).expect("Couldn't open file");

    let reader = BufReader::new(file);

    let header_initialized = false;
    let mut clauses = -1;
    let mut variables = -1;
    for line in reader.lines() {
        let line = line?;
        if line.chars().nth(0).unwrap() == 'c' {
            continue
        }

        if line.chars().nth(0).unwrap() == '%' {
            break
        }

        let splits: Vec<&str> = line.split_whitespace().collect();

        if splits[0] == "p" {
            if header_initialized {
                return Err("Header initialized multiple times".into());
            }
            println!("{}",line);
            println!("{}",splits.len());
            for item in &splits {
                dbg!(item);
            }
            if splits.len() != 4{
                return Err("Bad header format".into());
            }

            if splits[1] != "cnf" {
                return Err("Header does not indicate cnf".into());
            }

            let parsed_variables = splits[2].parse::<i32>();
            match parsed_variables {
                Ok(val) => if val > 0 {
                    variables = val
                } else {
                    return Err("Variable count cannot be negative".into())
                }
                Err(_) => return Err(format!("Variable count must be a number").into())
            }

            let parsed_clauses = splits[3].parse::<i32>();
            match parsed_clauses {
                Ok(val) => if val > 0 {
                    clauses = val
                } else {
                    return Err("Clause count cannot be negative".into())
                }
                Err(_) => return Err(format!("Clause count must be a number").into())
            }
            continue
        }

        for num in splits {
            let parsed_num = num.parse::<i32>();
            match parsed_num {
                Ok(val) => println!("Yay: {}", val),
                Err(_) => return Err(format!("{num} is not a number").into())
            };
        }

        println!("{} {}", variables, clauses)
    }
    Ok(())
}