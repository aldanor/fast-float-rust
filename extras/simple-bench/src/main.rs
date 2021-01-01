mod random;

use std::fs;
use std::iter;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use fastrand::Rng;
use lexical::FromLexical;
use structopt::StructOpt;

use fast_float::Float;

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
        #[structopt(short, default_value = "100000")]
        number: usize,
        /// Random generator seed
        #[structopt(short, default_value = "0")]
        seed: u64,
        /// Also save the generated inputs to file
        #[structopt(short = "f", parse(from_os_str))]
        filename: Option<PathBuf>,
    },
}

#[derive(Debug, Clone)]
struct BenchResult {
    pub name: String,
    pub times: Vec<i64>,
}

fn run_one_bench<T: Float, F: Fn(&str) -> T>(
    name: &str,
    inputs: &[String],
    repeat: usize,
    func: F,
) -> BenchResult {
    let mut times = Vec::with_capacity(repeat);
    let mut dummy = T::default();
    for _ in 0..repeat {
        let t0 = Instant::now();
        for input in inputs {
            dummy = dummy + func(input.as_str());
        }
        times.push(t0.elapsed().as_nanos() as _);
    }
    assert_ne!(dummy, T::default());
    times.sort();
    let name = name.into();
    BenchResult { name, times }
}

fn run_all_benches<T: Float + FromLexical + FromStr>(
    inputs: &[String],
    repeat: usize,
) -> Vec<BenchResult> {
    let ff_func = |s: &str| {
        fast_float::parse_float::<T>(s.as_bytes())
            .unwrap_or_default()
            .0
    };
    let ff_res = run_one_bench("fast_float", inputs, repeat, ff_func);

    let lex_func = |s: &str| {
        lexical_core::parse_partial::<T>(s.as_bytes())
            .unwrap_or_default()
            .0
    };
    let lex_res = run_one_bench("lexical_core", inputs, repeat, lex_func);

    let std_func = |s: &str| s.parse::<T>().unwrap_or_default();
    let std_res = run_one_bench("from_str", inputs, repeat, std_func);

    vec![ff_res, lex_res, std_res]
}

fn print_report(inputs: &[String], results: &[BenchResult], inputs_name: &str, ty: &str) {
    let n = inputs.len();
    let mb = (inputs.iter().map(|s| s.len()).sum::<usize>() as f64) / 1024. / 1024.;

    let width = 76;
    println!("{:=<width$}", "", width = width + 4);
    println!(
        "| {:^width$} |",
        format!("{} ({}, {:.2} MB, {})", inputs_name, n, mb, ty),
        width = width
    );
    println!("|{:=<width$}|", "", width = width + 2);
    let n = n as f64;
    print_table("ns/float", results, width, |t| t / n);
    print_table("Mfloat/s", results, width, |t| 1e3 * n / t);
    print_table("MB/s", results, width, |t| mb * 1e9 / t);
    println!("|{:width$}|", "", width = width + 2);
    println!("{:=<width$}", "", width = width + 4);
}

fn print_table(title: &str, results: &[BenchResult], width: usize, transform: impl Fn(f64) -> f64) {
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
    print!("| {:<h$}", title, h = h);
    for (name, _) in columns {
        print!("{:>w$}", name, w = w);
    }
    println!(" |");
    println!("|{:-<width$}|", "", width = width + 2);
    for res in results {
        print!("| {:<h$}", res.name, h = h);
        for &(_, idx) in columns {
            print!("{:>w$.2}", transform(res.times[idx] as f64), w = w);
        }
        println!(" |");
    }
}

fn main() {
    let opt: Opt = StructOpt::from_args();
    let (inputs, inputs_name) = match opt.command {
        Cmd::File { filename } => (
            fs::read_to_string(&filename)
                .unwrap()
                .trim()
                .lines()
                .map(String::from)
                .collect::<Vec<_>>(),
            filename.to_str().unwrap().to_owned(),
        ),
        Cmd::Random {
            gen,
            number,
            seed,
            filename,
        } => {
            let mut rng = Rng::with_seed(seed);
            let inputs: Vec<String> = iter::repeat_with(|| gen.gen(&mut rng))
                .take(number)
                .collect();
            if let Some(filename) = filename {
                fs::write(filename, inputs.join("\n")).unwrap();
            }
            (inputs, format!("{}", gen))
        }
    };
    let repeat = opt.repeat.max(1);
    let (results, ty) = if opt.float32 {
        (run_all_benches::<f32>(&inputs, repeat), "f32")
    } else {
        (run_all_benches::<f64>(&inputs, repeat), "f64")
    };
    print_report(&inputs, &results, &inputs_name, ty);
}
