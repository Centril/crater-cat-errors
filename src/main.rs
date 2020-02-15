use std::collections::BTreeMap as Map;
use std::fs;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};

use log::{debug, info};

fn main() -> io::Result<()> {
    env_logger::init();
    debug!("main initialized");

    let mut args = std::env::args();
    drop(args.next());
    let mut regressions = Map::new();
    let mut report_path = Path::new("report.md");
    for arg in args {
        if !arg.contains('=') {
            report_path = Path::new(arg);
            break;
        }
        let mut parts = arg.splitn(2, '=');
        let crater_run_name = parts.next().expect("first part before =").to_string();
        let analyze_dir = Path::new(parts.next().expect("second part after ="));

        regressions.extend(collect_regression_paths(
            crater_run_name.clone(),
            &analyze_dir.join("gh"),
        )?);
        regressions.extend(collect_regression_paths(
            crater_run_name.clone(),
            &analyze_dir.join("reg"),
        )?);
    }
    debug!("the regression map is: {:#?}", regressions);

    let errors = collect_errors(regressions)?;
    debug!("the error map is: {:#?}", errors);

    let report = generate_report(errors);
    debug!("the report is:\n{}", report);

    std::fs::write(&report_path, report)?;

    Ok(())
}

fn generate_report(errors: ErrorMap) -> String {
    errors
        .iter()
        .map(|(error, krates)| {
            let mut lines = vec![
                format!("### {}", error),
                format!("Number of crates regressed: {}", krates.len()),
                "<details>\n".to_string(),
            ];
            lines.extend(krates.iter().map(|(run, (n, v))| {
                if v.chars().next().unwrap().is_digit(10) {
                    // crates.io
                    format!(
                        "- [`{} v{}`](https://crater-reports.s3.amazonaws.com/{}/reg/{}-{}/log.txt)",
                        n, v, run, n, v
                    )
                } else {
                    // github
                    format!(
                        "- [`{}/{}`](https://crater-reports.s3.amazonaws.com/{}/gh/{}.{}/log.txt)",
                        n, v, run, n, v
                    )
                }
            }));
            lines.push("\n</details>".to_string());
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

type ErrorMap = Map<String, Vec<(String, Crate)>>;

fn collect_errors(regressions: RegressionMap) -> io::Result<ErrorMap> {
    let mut map: ErrorMap = Map::new();
    for (krate, (crater_run, path)) in regressions {
        info!("processing regression {}-{}: {:#?}", krate.0, krate.1, path);
        let contents = read_file_to_string(&path)?;
        let errors = process_regression_file(&contents);
        for error in errors {
            map.entry(error.to_string())
                .or_default()
                .push((crater_run.clone(), krate.clone()));
        }
    }
    Ok(map)
}

fn process_regression_file(contents: &str) -> Vec<String> {
    let mut contents = contents
        .lines()
        .filter(|l| l.starts_with("[INFO] [stderr]"))
        .map(|l| l.trim_start_matches("[INFO] [stderr]").trim())
        .filter(|l| l.starts_with("error:") || l.starts_with("error["))
        .filter(|l| !l.starts_with("error: aborting"))
        .filter(|l| !l.starts_with("error: could not compile"))
        .filter(|l| !l.starts_with("error: Compilation failed, aborting rustdoc"))
        .filter(|l| !l.starts_with("error: Could not document"))
        .filter(|l| !l.starts_with("error: build failed"))
        .filter(|l| !l.starts_with("error: the lock file"))
        .map(|l| erase_backtick_contents(l))
        .collect::<Vec<_>>();

    contents.sort();
    contents.dedup();

    debug!("filtered regression contents: {:#?}", contents);
    contents
}

fn erase_backtick_contents(line: &str) -> String {
    let mut newline = String::new();
    let mut in_backticks = false;
    for ch in line.chars() {
        match ch {
            '`' => {
                if in_backticks {
                    newline.push_str("...");
                }
                in_backticks = !in_backticks;
                newline.push(ch);
            }
            _ => {
                if !in_backticks {
                    newline.push(ch);
                }
            }
        }
    }
    newline
}

fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut contents = String::new();
    io::BufReader::new(fs::File::open(path)?).read_to_string(&mut contents)?;
    Ok(contents)
}

type Crate = (String, String);

type RegressionMap = Map<Crate, (String, PathBuf)>;

fn collect_regression_paths(name: String, dir: &Path) -> io::Result<RegressionMap> {
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
                if file_name.to_str().unwrap().starts_with("beta-")
                    || file_name.to_str().unwrap().starts_with("try")
                {
                    paths.insert(
                        (crate_name.clone(), crate_version.clone()),
                        (name.clone(), path),
                    );
                }
            }
        }
    }
    info!("total regression files = {}", paths.len());

    Ok(paths)
}
