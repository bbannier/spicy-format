use std::path::Path;

fn main() {
    let is_git_repo = Path::new(".git").exists();
    if is_git_repo {
        vergen::EmitBuilder::builder()
            .idempotent()
            .git_describe(true, true, None)
            .emit()
            .unwrap();
    }
}
