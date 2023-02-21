use similar::{DiffOp, TextDiff};
use std::cmp::Ordering;
use std::error::Error;
use std::fs;
use std::fs::DirEntry;

fn main() -> Result<(), Box<dyn Error>> {
    let mut argv = std::env::args();
    let _argv0 = argv.next();

    let git_dir = argv.next().unwrap();
    let mips_dir = argv.next().unwrap_or_else(|| "/tmp/mips".to_owned());

    let dir = fs::read_dir(mips_dir)?;

    let game_mips: Vec<_> = dir.filter_map(|de| de.ok()).collect();

    let git_mips: Vec<_> = fs::read_dir(git_dir)?
        .filter_map(|de| de.ok())
        .filter(|de| {
            let x = de
                .path()
                .to_str()
                .map_or(false, |path| path.ends_with(".mips"));
            x
        })
        .filter_map(|de| fs::read_to_string(de.path()).ok().map(|str| (de, str)))
        .collect();

    let mut results: Vec<_> = game_mips
        .iter()
        .map(|mips1| (mips1, find_git_match(mips1, &git_mips)))
        .collect();

    results.sort_by(
        |(_mips_a, result_a), (_mips_b, result_b)| match (result_a, result_b) {
            (Err(_), Err(_)) => Ordering::Equal,
            (Err(_), Ok(_)) => Ordering::Less,
            (Ok(_), Err(_)) => Ordering::Greater,
            (Ok((_git_file_a, a_score)), Ok((_git_file_b, b_score))) => b_score.cmp(a_score),
        },
    );

    for result in results {
        println!("{:?}", result)
    }

    Ok(())
}

fn find_git_match<'a>(
    game: &'a DirEntry,
    git: &'a [(DirEntry, String)],
) -> Result<(&'a DirEntry, usize), std::io::Error> {
    let src_a = fs::read_to_string(game.path())?;

    let tmp = git
        .iter()
        .map(|(de, src_b)| compute_score(&src_a, de, src_b))
        .fold(None, |a, b| match a {
            None => Some(b),
            Some(a) => {
                if a.1 <= b.1 {
                    Some(a)
                } else {
                    Some(b)
                }
            }
        });

    Ok(tmp.unwrap())
}

fn compute_score<'a>(
    src_a: &'_ str,
    file_b: &'a DirEntry,
    src_b: &'_ str,
) -> (&'a DirEntry, usize) {
    let diff = TextDiff::from_lines(src_a, src_b);

    let changes = diff.ops().iter().map(diff_cost).sum();

    (file_b, changes)
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
