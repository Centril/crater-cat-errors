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

The script below, as of this writing, will do this (where `RUN_ID` is the crater
run name). `PART_ID` can be found by going to any regression link; it is the
`end=` parameter to crater (e.g., `beta-2020-02-05` for the 1.42 crater run).

```bash
RUN_ID=beta-1.42-1
PART_ID=beta-2020-02-05
wget https://crater-reports.s3.amazonaws.com/$RUN_ID/logs-archives/regressed.tar.gz
tar xf regressed.tar.gz
RUST_LOG=info cargo run -- $RUN_ID/$PART_ID=./regressed
```

Your report will now be in `report.md`.

Have fun with the triage! =)
