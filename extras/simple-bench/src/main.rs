mod random;

use std::fs;
use std::iter;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use fastrand::Rng;
use lexical::FromLexical;
use structopt::StructOpt;

use fast_float::FastFloat;

use random::RandomGen;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "fast-float-simple-bench",
    about = "fast-float benchmark utility",
    no_version
)]
struct Opt {
    /// Parse numbers as float32 (default is float64)
    #[structopt(short, long = "32")]
    float32: bool,
    /// How many times to repeat parsing
    #[structopt(short, default_value = "1000")]
    repeat: usize,
    /// Only run fast-float benches
    #[structopt(short)]
    only_fast_float: bool,
    #[structopt(subcommand)]
    command: Cmd,
}

#[derive(Debug, StructOpt)]
enum Cmd {
    /// Read the floats from file
    File {
        /// Input file (one number per line)
        #[structopt(parse(from_os_str))]
        filename: PathBuf,
    },
    /// Generate random floats in (0, 1]
    Random {
        /// Random generator to be used
        #[structopt(
            default_value = "uniform",
            parse(try_from_str),
            possible_values = RandomGen::variants()
        )]
        gen: RandomGen,
        /// Number of random floats generated
        #[structopt(short = "n", default_value = "50000")]
        count: usize,
        /// Random generator seed
        #[structopt(short, default_value = "0")]
        seed: u64,
        /// Also save the generated inputs to file
        #[structopt(short = "f", parse(from_os_str))]
        filename: Option<PathBuf>,
    },
    /// Run all benchmarks for fast-float only
    All {
        /// Number of random floats generated
        #[structopt(short = "n", default_value = "50000")]
        count: usize,
        /// Random generator seed
        #[structopt(short, default_value = "0")]
        seed: u64,
    },
}

#[derive(Debug, Clone)]
struct BenchResult {
    pub name: String,
    pub times: Vec<i64>,
    pub count: usize,
    pub bytes: usize,
}

fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = core::ptr::read_volatile(&dummy);
        core::mem::forget(dummy);
        ret
    }
}

fn run_bench<T: FastFloat, F: Fn(&str) -> T>(
    inputs: &[String],
    repeat: usize,
    func: F,
) -> Vec<i64> {
    const WARMUP: usize = 1000;
    let mut times = Vec::with_capacity(repeat + WARMUP);
    for _ in 0..repeat + WARMUP {
        let t0 = Instant::now();
        for input in inputs {
            black_box(func(input.as_str()));
        }
        times.push(t0.elapsed().as_nanos() as _);
    }
    times.split_at(WARMUP).1.into()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Method {
    FastFloat,
    #[cfg(feature = "use_tokenized")]
    FastFloatTokenized,
    Lexical,
    FromStr,
}

fn type_str(float32: bool) -> &'static str {
    if float32 {
        "f32"
    } else {
        "f64"
    }
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn parse_sign<'a>(s: &'a str) -> (bool, &'a str) {
    match s.as_bytes().get(0) {
        Some(&b'+') => (false, &s[1..]),
        Some(&b'-') => (true, &s[1..]),
        _ => (false, s),
    }
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn decimal_index(s: &str) -> Option<usize> {
    s.as_bytes().iter().position(|&c| c == b'.')
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn exponent_index(s: &str) -> Option<usize> {
    s.as_bytes().iter().position(|&c| c == b'e' || c == b'E')
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn split_index<'a>(s: &'a str, index: usize) -> (&'a str, &'a str) {
    let (lead, trail) = s.as_bytes().split_at(index);
    let trail = &trail[1..];
    use std::str;
    unsafe {
        (str::from_utf8_unchecked(lead), str::from_utf8_unchecked(trail))
    }
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn split_end<'a>(s: &'a str) -> (&'a str, &'a str) {
    let (lead, trail) = s.as_bytes().split_at(s.len());
    use std::str;
    unsafe {
        (str::from_utf8_unchecked(lead), str::from_utf8_unchecked(trail))
    }
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn parse_exponent(s: &str) -> i64 {
    s.parse::<i64>().unwrap()
}

#[inline]
#[cfg(feature = "use_tokenized")]
fn tokenize<'a>(s: &'a str) -> (&'a str, &'a str, i64, bool) {
    let (negative, s) = parse_sign(s);
    if let Some(index) = decimal_index(s) {
        let (i, rest) = split_index(s, index);
        if let Some(index) = exponent_index(s) {
            let (f, exp) = split_index(rest, index);
            let exp = parse_exponent(exp);
            (i, f, exp, negative)
        } else {
            (i, rest, 0, negative)
        }
    } else {
        if let Some(index) = exponent_index(s) {
            let (i, exp) = split_index(s, index);
            let (i, f) = split_end(i);
            let exp = parse_exponent(exp);
            (i, f, exp, negative)
        } else {
            let (i, f) = split_end(s);
            (i, f, 0, negative)
        }
    }
}

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FastFloat => "fast-float",
            #[cfg(feature = "use_tokenized")]
            Self::FastFloatTokenized => "fast-float-tokenized",
            Self::Lexical => "lexical",
            Self::FromStr => "from_str",
        }
    }

    fn run_as<T: FastFloat + FromLexical + FromStr>(
        &self,
        input: &Input,
        repeat: usize,
        name: &str,
    ) -> BenchResult {
        let data = &input.data;
        let times = match self {
            Self::FastFloat => run_bench(data, repeat, |s: &str| {
                fast_float::parse_partial::<T, _>(s).unwrap_or_default().0
            }),
            #[cfg(feature = "use_tokenized")]
            Self::FastFloatTokenized => run_bench(data, repeat, |s: &str| {
                let (i, f, e, n) = tokenize(s);
                fast_float::parse_from_parts::<T, _>(i, f, e, n)
            }),
            Self::Lexical => run_bench(data, repeat, |s: &str| {
                lexical_core::parse_partial::<T>(s.as_bytes())
                    .unwrap_or_default()
                    .0
            }),
            Self::FromStr => run_bench(data, repeat, |s: &str| s.parse::<T>().unwrap_or_default()),
        };

        BenchResult {
            times,
            name: name.into(),
            count: input.count(),
            bytes: input.bytes(),
        }
    }

    pub fn run(&self, input: &Input, repeat: usize, name: &str, float32: bool) -> BenchResult {
        if float32 {
            self.run_as::<f32>(input, repeat, name)
        } else {
            self.run_as::<f64>(input, repeat, name)
        }
    }

    pub fn all() -> &'static [Self] {
        #[cfg(feature = "use_tokenized")]
        {
            &[Method::FastFloat, Method::FastFloatTokenized, Method::Lexical, Method::FromStr]
        }

        #[cfg(not(feature = "use_tokenized"))]
        {
            &[Method::FastFloat, Method::Lexical, Method::FromStr]
        }
    }
}

fn print_report(results: &[BenchResult], title: &str) {
    let width = 81;
    println!("{:=<width$}", "", width = width + 4);
    println!("| {:^width$} |", title, width = width);
    println!("|{:=<width$}|", "", width = width + 2);
    print_table("ns/float", results, width, |t, n, _| t as f64 / n as f64);
    print_table("Mfloat/s", results, width, |t, n, _| {
        1e3 * n as f64 / t as f64
    });
    print_table("MB/s", results, width, |t, _, b| {
        b as f64 * 1e9 / 1024. / 1024. / t as f64
    });
    println!("|{:width$}|", "", width = width + 2);
    println!("{:=<width$}", "", width = width + 4);
}

fn print_table(
    heading: &str,
    results: &[BenchResult],
    width: usize,
    transform: impl Fn(i64, usize, usize) -> f64,
) {
    let repeat = results[0].times.len();
    let columns = &[
        ("min", 0),
        ("5%", repeat / 20),
        ("25%", repeat / 4),
        ("median", repeat / 2),
        ("75%", (3 * repeat) / 4),
        ("95%", (19 * repeat) / 20),
        ("max", repeat - 1),
    ];
    let w = 9;
    let h = width - 7 * w;

    println!("|{:width$}|", "", width = width + 2);
    print!("| {:<h$}", heading, h = h);
    for (name, _) in columns {
        print!("{:>w$}", name, w = w);
    }
    println!(" |");
    println!("|{:-<width$}|", "", width = width + 2);
    for res in results {
        print!("| {:<h$}", res.name, h = h);
        let (n, b) = (res.count, res.bytes);
        let mut metrics = res
            .times
            .iter()
            .map(|&t| transform(t, n, b))
            .collect::<Vec<_>>();
        metrics.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for &(_, idx) in columns {
            print!("{:>w$.2}", metrics[idx], w = w);
        }
        println!(" |");
    }
}

struct Input {
    pub data: Vec<String>,
    pub name: String,
}

impl Input {
    pub fn from_file(filename: impl AsRef<Path>) -> Self {
        let filename = filename.as_ref();
        let data = fs::read_to_string(&filename)
            .unwrap()
            .trim()
            .lines()
            .map(String::from)
            .collect();
        let name = filename.file_name().unwrap().to_str().unwrap().into();
        Self { data, name }
    }

    pub fn from_random(gen: RandomGen, count: usize, seed: u64) -> Self {
        let mut rng = Rng::with_seed(seed);
        let data = iter::repeat_with(|| gen.gen(&mut rng))
            .take(count)
            .collect();
        let name = format!("{}", gen);
        Self { data, name }
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn bytes(&self) -> usize {
        self.data.iter().map(|s| s.len()).sum()
    }

    pub fn title(&self, float32: bool) -> String {
        format!(
            "{} ({}, {:.2} MB, {})",
            self.name,
            self.count(),
            self.bytes() as f64 / 1024. / 1024.,
            type_str(float32),
        )
    }
}

fn main() {
    let opt: Opt = StructOpt::from_args();

    let methods = if !opt.only_fast_float && !matches!(&opt.command, &Cmd::All {..}) {
        Method::all().into()
    } else {
        vec![Method::FastFloat]
    };

    let inputs = match opt.command {
        Cmd::File { filename } => vec![Input::from_file(filename)],
        Cmd::Random {
            gen,
            count,
            seed,
            filename,
        } => {
            let input = Input::from_random(gen, count, seed);
            if let Some(filename) = filename {
                fs::write(filename, input.data.join("\n")).unwrap();
            }
            vec![input]
        }
        Cmd::All { count, seed } => {
            let mut inputs = vec![];
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext/data");
            inputs.push(Input::from_file(data_dir.join("mesh.txt")));
            inputs.push(Input::from_file(data_dir.join("canada.txt")));
            for &gen in RandomGen::all() {
                inputs.push(Input::from_random(gen, count, seed))
            }
            inputs
        }
    };

    let mut results = vec![];
    for input in &inputs {
        for method in &methods {
            let name = if inputs.len() == 1 {
                method.name()
            } else {
                &input.name
            };
            results.push(method.run(input, opt.repeat.max(1), name, opt.float32));
        }
    }

    let title = if inputs.len() == 1 {
        inputs[0].title(opt.float32)
    } else {
        format!("fast-float (all, {})", type_str(opt.float32))
    };
    print_report(&results, &title);
}
