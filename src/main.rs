use std::collections::BTreeMap as Map;
use std::path::{Path, PathBuf};
use std::io::{self, Read as _, Write as _};
use std::fs;

use log::{info, debug};

fn main() -> io::Result<()> {
    env_logger::init();
    debug!("main initialized");

    let mut args = std::env::args();
    drop(args.next());
    let analyze_dir = args.next().unwrap();
    let analyze_dir = Path::new(&analyze_dir);
    let save_report_to = args.next().unwrap();
    let save_report_to = Path::new(&save_report_to);

    let regressions = collect_regression_paths(analyze_dir)?;
    debug!("the regression map is: {:#?}", regressions);

    let errors = collect_errors(regressions)?;
    debug!("the error map is: {:#?}", errors);

    let report = generate_report(errors);
    debug!("the report is:\n{}", report);

    let mut out_file = fs::File::create(save_report_to)?;
    write!(&mut out_file, "{}", report)?;

    Ok(())
}

fn generate_report(errors: ErrorMap) -> String {
    errors.iter()
        .map(|(error, krates)| {
            let mut lines = vec![
                format!("### {}", error),
                format!("Number of crates regressed: {}", krates.len()),
                "<details>".to_string(),
            ];
            lines.extend(krates.iter().map(|(n, v)| format!("- `{}-{}`", n, v)));
            lines.push("</details>".to_string());
            lines.join("\n\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

type ErrorMap = Map<String, Vec<Crate>>;

fn collect_errors(regressions: RegressionMap) -> io::Result<ErrorMap> {
    let mut map: ErrorMap = Map::new();
    for (krate, path) in regressions {
        info!("processing regression {}-{}: {:#?}", krate.0, krate.1, path);
        let contents = read_file_to_string(&path)?;
        let errors = process_regression_file(&contents);
        for error in errors {
            map.entry(error.to_string())
                .or_default()
                .push(krate.clone());
        }
    }
    Ok(map)
}

fn process_regression_file(contents: &str) -> Vec<&str> {
    let mut contents = contents.lines()
        .filter(|l| l.starts_with("[INFO] [stderr]"))
        .map(|l| l.trim_start_matches("[INFO] [stderr]").trim())
        .filter(|l| l.starts_with("error:") || l.starts_with("error["))
        .filter(|l| !l.starts_with("error: aborting"))
        .filter(|l| !l.starts_with("error: Could not compile"))
        .filter(|l| !l.starts_with("error: build failed"))
        .filter(|l| !l.starts_with("error: the lock file /mnt"))
        .collect::<Vec<_>>();

    contents.sort();
    contents.dedup();

    debug!("filtered regression contents: {:#?}", contents);
    contents
}

fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut contents = String::new();
    io::BufReader::new(fs::File::open(path)?).read_to_string(&mut contents)?;
    Ok(contents)
}

type Crate = (String, String);

type RegressionMap = Map<Crate, PathBuf>;

fn collect_regression_paths(dir: &Path) -> io::Result<RegressionMap> {
    let mut paths = Map::new();

    info!("collecting paths in: {}", dir.display());
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        assert!(path.is_dir());
        let crate_name = entry.file_name().into_string().unwrap();
        info!("found crate = {}", crate_name);

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            assert!(path.is_dir());
            let crate_version = entry.file_name().into_string().unwrap();
            debug!("found version of crate {} = {}", crate_name, path.display());

            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                assert!(!path.is_dir());
                let file_name = entry.file_name();
                if file_name.to_str().unwrap().starts_with("try") {
                    paths.insert((crate_name.clone(), crate_version.clone()), path);
                }
            }
        }
    }
    info!("total regression files = {}", paths.len());

    Ok(paths)
}
