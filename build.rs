use std::path::Path;

fn main() {
    let is_git_repo = Path::new(".git").exists();
    assert!(is_git_repo, "foo");
    vergen::EmitBuilder::builder()
        .idempotent()
        .git_describe(true, true, None)
        .emit()
        .unwrap();
}
