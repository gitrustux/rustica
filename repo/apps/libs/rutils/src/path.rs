// File path utilities

pub fn canonicalize(path: &str) -> Result<String, ()> {
    // TODO: Implement path canonicalization
    Ok(path.to_string())
}

pub fn join(base: &str, path: &str) -> String {
    // TODO: Implement path joining
    format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
}
