# Device driver UI tests

A crate containing the high level test suite.
To run the tests, run `cargo test` or `cargo nextest run`.

All tests are in the [cases](./cases/) folder. The tests are discovered by the build.rs script of this crate.
The tests have an input file and a known output file.
The input file is generated to an output and the result is compared to the known output.
If they're not the same, the test fails with a diff view.

If they're the same and if the output is a Rust file, it will be compiled as a cargo script.
If the compilation does not succeed without warning, the test also fails.

In the case where there are a bunch of good changes that should be committed,
you can run `cargo run -- accept` on this crate to accept the changes.
The output files will then be updated with the current output of the generation.
