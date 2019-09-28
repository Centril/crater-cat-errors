# `crater-cat-errors`

[crater]: http://crater.rust-lang.org/
[pr-63247]: https://github.com/rust-lang/rust/pull/63247
[completion message]: https://github.com/rust-lang/rust/pull/63247#issuecomment-526870105
[full-report]: https://crater-reports.s3.amazonaws.com/pr-63247/index.html
[downloads]: https://crater-reports.s3.amazonaws.com/pr-63247/downloads.html
[regressed crates]: https://crater-reports.s3.amazonaws.com/pr-63247/logs-archives/regressed.tar.gz

This is a small program to assist you in triaging [crater] regressions when doing
a crater run of a series of changes batched together. It allows you to see what
the unique error messages were and how many crates regressed for each message.

To run the program, for example on [pr-63247], first go to the @craterbot's
[completion message], and then [open the full report][full-report]. Once there,
go to the [downloads] section, click on [regressed crates] to get the
`regressed.tar.gz` file. Now extract the file into the directory `regressed`.
Inside `my_dir` there should be `reg` and `gh` for crates.io and GitHub respectively.
Once done, compile and run `crater-cat-errors` using:

```
cargo build && RUST_LOG=info ./target/debug/crater-cat-errors ./regressed/reg report.md
```

Your report will now be in `report.md`.

Have fun with the triage! =)
