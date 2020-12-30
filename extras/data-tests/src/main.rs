use std::fs::{read_dir, File};
use std::io::{BufRead, BufReader};
use std::mem::transmute;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct TestCase {
    float32: f32,
    float64: f64,
    string: String,
}

impl TestCase {
    pub fn parse(string: String) -> Self {
        let float32 = unsafe { transmute(u32::from_str_radix(&string[5..13], 16).unwrap()) };
        let float64 = unsafe { transmute(u64::from_str_radix(&string[14..30], 16).unwrap()) };
        let string = string[31..].to_string();
        Self {
            float32,
            float64,
            string,
        }
    }

    fn execute_one<F: fast_float::Float>(&self, expected: F) {
        let s = self.string.as_bytes();

        let r = fast_float::parse_float::<F>(s);
        if !r.is_some() {
            dbg!(self);
            eprintln!("Failed to parse as f32: {:?}", self.string);
        }
        let (value, rest) = r.unwrap();
        if !rest.is_empty() || value != expected {
            if !rest.is_empty() {
                eprintln!("Expected empty string remainder, got: {:?}", rest);
            }
            if value != expected {
                eprintln!("Expected output {}, got {}", expected, value);
            }
            panic!("Test case failed: {:?}", self);
        }
    }

    pub fn execute(&self) {
        self.execute_one::<f32>(self.float32);
        self.execute_one::<f64>(self.float64);
    }
}

fn parse_test_file(filename: impl AsRef<Path>) -> impl Iterator<Item = TestCase> {
    let file = File::open(filename).unwrap();
    BufReader::new(file)
        .lines()
        .map(Result::unwrap)
        .map(TestCase::parse)
}

fn run_test_cases(filename: impl AsRef<Path>) -> usize {
    parse_test_file(filename).map(|test| test.execute()).count()
}

fn test_file_paths() -> Vec<PathBuf> {
    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext/data");
    let mut paths = read_dir(&test_data_dir)
        .unwrap()
        .map(|d| d.unwrap().path())
        .filter(|p| {
            p.is_file()
                && p.extension().map(|e| e.to_str()) == Some(Some("txt"))
                && p.file_name().map(|f| f.to_str()) != Some(Some("CMakeLists.txt"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn main() {
    for path in &test_file_paths() {
        eprint!("Running test cases in {:?}... ", path.file_name().unwrap());
        let count = run_test_cases(path);
        eprintln!("{} tests passed.", count);
    }
}
