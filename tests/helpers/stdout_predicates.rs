use predicates::str as p_str;
use std::path::Path;

pub fn contains_read_error(
    file_path: impl AsRef<Path>,
    error_str: &str,
) -> impl predicates::Predicate<str> {
    p_str::contains(format!(
        "error: could not read `{}`: {error_str}",
        file_path.as_ref().display(),
    ))
}

pub fn contains_no_such_file_error(file_path: impl AsRef<Path>) -> impl predicates::Predicate<str> {
    contains_read_error(file_path, "No such file or directory (os error 2)")
}

pub fn contains_graphql_file_error_without_location(
    file_path: impl AsRef<Path>,
) -> impl predicates::Predicate<str> {
    p_str::contains(format!("--> {}", file_path.as_ref().display()))
}

pub fn contains_graphql_file_error_with_location(
    file_path: impl AsRef<Path>,
    (line, col): (usize, usize),
) -> impl predicates::Predicate<str> {
    p_str::contains(format!("--> {}:{line}:{col}", file_path.as_ref().display()))
}
