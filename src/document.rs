use std::fs;
use regex::Regex;

// Struct that represents a row, more information will be added in later updates
#[derive(Clone)]
pub struct Row {
    pub string: String
}

impl Row {
    pub fn from(string: &str) -> Row {
        Row {
            string: string.to_string()
        }
    }
}

#[derive(Clone)]
pub struct Doc {
    pub rows: Vec<Row>,
    pub path: String,
    pub wrap: bool,
}

impl Doc {
    pub fn new() -> Doc {
        Doc {
            rows: vec![],
            path: format!(""),
            wrap: true
        }
    }

    pub fn open(&mut self, path: String) {
        let file = fs::read_to_string(&path).unwrap();
        let rows = Doc::split_file(&file);

        self.path = path;
        self.rows = rows.iter().map(|row| Row::from(*row)).collect();
    }

    pub fn split_file(contents: &str) -> Vec<&str> {
        let splitter = Regex::new("(?ms)(\r\n|\n)").unwrap();
        splitter.split(contents).collect()
    }
}
