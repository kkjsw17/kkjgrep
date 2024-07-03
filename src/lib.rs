use std::error::Error;
use std::{env, fs};

#[derive(Debug)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub case_sensitive: bool,
    pub line_number: bool,
    pub replace: String,
}

impl Config {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Config, &'static str> {
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        let args: Vec<String> = args.collect();

        let case_sensitive = Self::find_case_sensitive(&args);
        // if case_sensitive is true, ignore_case is false
        let ignore_case = !case_sensitive && Self::find_ignore_case(&args);
        let line_number = Self::find_line_number(&args);
        let replace = Self::find_replace(args);

        Ok(Config {
            query,
            file_path,
            case_sensitive,
            ignore_case,
            line_number,
            replace,
        })
    }

    fn find_case_sensitive(args: &Vec<String>) -> bool {
        args.iter().any(|arg| arg == "-s" || arg == "--case-sensitive")
    }

    fn find_ignore_case(args: &Vec<String>) -> bool {
        match env::var("IGNORE_CASE") {
            Ok(val) => val == "1",
            Err(_) => args.iter().any(|arg| arg == "-i" || arg == "--ignore-case"),
        }
    }

    fn find_line_number(args: &Vec<String>) -> bool {
        args.iter().any(|arg| arg == "-n" || arg == "--line-number")
    }

    fn find_replace(args: Vec<String>) -> String {
        args.iter()
            .position(|arg| arg == "-r" || arg == "--replace")
            .and_then(|pos| args.get(pos + 1))
            .unwrap_or(&String::new())
            .clone()
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;
    let results = search_based_on_case(&config, &contents);

    for (i, line) in results.iter().enumerate() {
        let line = replace_if_not_empty(&config, line);
        print_based_on_line_number(&config, i, line);
    }

    Ok(())
}

fn print_based_on_line_number(config: &Config, i: usize, line: String) {
    if config.line_number {
        println!("{}: {}", i + 1, line);
    } else {
        println!("{}", line);
    }
}

fn replace_if_not_empty(config: &Config, line: &&str) -> String {
    if !config.replace.is_empty() {
        replace(&config.query, &config.replace, line)
    } else {
        line.to_string()
    }
}

fn search_based_on_case<'a>(config: &'a Config, contents: &'a String) -> Vec<&'a str>  {
    if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    }
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

fn search_case_insensitive<'a>(
    query: &str,
    contents: &'a str,
) -> Vec<&'a str> {
    let query = query.to_lowercase();

    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

fn replace<'a>(query: &str, replace: &str, contents: &'a str, ) -> String {
    contents.replace(query, replace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}

