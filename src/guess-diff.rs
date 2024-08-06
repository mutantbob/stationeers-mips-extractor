use similar::{DiffOp, TextDiff};
use std::cmp::Ordering;
use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;

fn is_mips(de: &DirEntry) -> bool {
    de.path()
        .to_str()
        .map_or(false, |path| path.ends_with(".mips"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut argv = std::env::args();
    let _argv0 = argv.next();

    let git_dir = argv.next().unwrap();
    let mips_dir = argv.next().unwrap_or_else(|| "/tmp/mips".to_owned());

    let dir = fs::read_dir(mips_dir)?;

    let game_mips: Vec<_> = dir
        .filter_map(|de| de.map_or(None, |de| if is_mips(&de) { Some(de) } else { None }))
        .collect();

    let file_paths = recurse_find_mips(git_dir)?;
    let file_paths: Vec<_> = file_paths.collect();
    for path in file_paths.iter() {
        println!("{:?}", path.path());
    }
    let git_mips: Vec<_> = file_paths
        .into_iter()
        .filter_map(|de| fs::read_to_string(de.path()).ok().map(|str| (de, str)))
        .collect();

    let mut results: Vec<_> = game_mips
        .iter()
        .map(|mips1| (mips1, find_git_match(mips1, &git_mips)))
        .filter(|(_path, results)| match results {
            Err(_) => false,
            Ok(match1) => match1.diff_count > 0,
        })
        .collect();

    results.sort_by(
        |(_mips_a, result_a), (_mips_b, result_b)| match (result_a, result_b) {
            (Err(_), Err(_)) => Ordering::Equal,
            (Err(_), Ok(_)) => Ordering::Less,
            (Ok(_), Err(_)) => Ordering::Greater,
            (Ok(match_a), Ok(match_b)) => match_a.diff_count.cmp(&match_b.diff_count),
        },
    );

    for (unknown, result) in results {
        if let (Some(file_name), Ok(match1)) = (unknown.path().to_str(), result) {
            if let Some(file2_name) = match1.path {
                println!(
                    "diff -uw {} {} # {} {:.0}%",
                    file_name,
                    file2_name,
                    match1.diff_count,
                    100.0 * match1.diff_frac
                );
            }
        }
    }

    Ok(())
}

fn recurse_find_mips(
    git_dir: impl AsRef<Path>,
) -> Result<
    //Filter<FilterMap<ReadDir, fn(Result<DirEntry>) -> Option<DirEntry>>, fn(&DirEntry) -> bool { is_mips }>
    impl Iterator<Item = DirEntry>,
    Box<dyn Error>,
> {
    let file_paths = fs::read_dir(git_dir)?
        .flat_map(|de| de.ok())
        .flat_map(|de| recurse_find_mips_of_file(de).ok())
        .flatten();
    Ok(file_paths)
}

fn recurse_find_mips_of_file(
    de: DirEntry,
) -> Result<Box<dyn Iterator<Item = DirEntry>>, Box<dyn Error>> {
    Ok(if de.path().is_dir() {
        let tmp: Box<dyn Iterator<Item = DirEntry>> = Box::new(recurse_find_mips(de.path())?);
        tmp
    } else if is_mips(&de) {
        Box::new(std::iter::once(de))
    } else {
        Box::new(std::iter::empty())
    })
}

struct Match {
    path: Option<String>,
    diff_count: usize,
    diff_frac: f32,
}

impl Match {
    pub fn new(path: Option<String>, diff_count: usize, diff_frac: f32) -> Self {
        Self {
            path,
            diff_count,
            diff_frac,
        }
    }
}

fn find_git_match<'a>(
    game: &'a DirEntry,
    git: &'a [(DirEntry, String)],
) -> Result<Match, std::io::Error> {
    let src_a = fs::read_to_string(game.path())?;

    let tmp = git
        .iter()
        .map(|(de, src_b)| compute_score(&src_a, de, src_b))
        .fold(None, |a, b| match a {
            None => Some(b),
            Some(a) => {
                if a.diff_count <= b.diff_count {
                    Some(a)
                } else {
                    Some(b)
                }
            }
        });

    Ok(tmp.unwrap())
}

fn compute_score(src_a: &str, file_b: &DirEntry, src_b: &str) -> Match {
    let diff = TextDiff::from_lines(src_a, src_b);

    let changes = diff.ops().iter().map(diff_cost).sum();

    let a_count = count_lines(src_a);
    let b_count = count_lines(src_b);

    Match::new(
        file_b.path().to_str().map(|x| x.to_string()),
        changes,
        changes as f32 / (a_count.max(b_count) as f32),
    )
}

fn count_lines(src_a: &str) -> usize {
    src_a.chars().filter(|&ch| ch == '\n').count()
}

fn diff_cost(op: &DiffOp) -> usize {
    match op {
        DiffOp::Equal { .. } => 0,
        DiffOp::Delete { old_len, .. } => *old_len,
        DiffOp::Insert { new_len, .. } => *new_len,
        DiffOp::Replace {
            old_len, new_len, ..
        } => (*old_len).max(*new_len),
    }
}
