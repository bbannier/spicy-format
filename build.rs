use std::path::Path;

fn main() {
    let is_git_repo = Path::new(".git").exists();
    if is_git_repo {
        let git2 = vergen_git2::Git2Builder::default()
            .describe(true, true, None)
            .build()
            .unwrap();
        vergen_git2::Emitter::new()
            .add_instructions(&git2)
            .unwrap()
            .emit()
            .unwrap();
    }
}
