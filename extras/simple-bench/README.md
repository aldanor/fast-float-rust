This crate provides a utility for benchmarking the `fast-float` crate against
`lexical_core` and standard library's `FromStr`.

To run a file-based test:

```sh
cargo run --release -- file ext/canada.txt
```

There's two files used in benchmarking of the original fast_float C++ library
(canada.txt and mesh.txt), they are sourced from
[this](https://github.com/lemire/simple_fastfloat_benchmark) repository. These
files can be found under `ext/data`.

To run a randomized test:

```sh
cargo run --release -- random uniform
```

For more details and options (choosing a different random generator, storing 
randomized inputs to a file, changing the number of runs, or switching between 
32-bit and 64-bit floats), refer to help:

```
cargo run --release -- --help
```
