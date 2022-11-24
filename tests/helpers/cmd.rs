use assert_cmd::assert::Assert;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::str as p_str;
use predicates::Predicate;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_QLCRC_JSON_PATH: &str = ".qlcrc.json";
const FIXTURE_ROOT_PATH: &str = "tests/fixtures";
const TS_FILE_HEADER: &str = "/* eslint-disable */
// This file was automatically generated and should not be edited.

";

/// Predicate that never fails and overwrites expected fixtures with actual output
#[derive(Debug)]
struct FixtureSaverPredicate<'a, P> {
    wrapping_predicate: P,
    original_path: &'a Path,
}

impl<'a, P: Predicate<str>> FixtureSaverPredicate<'a, P> {
    fn new(wrapping_predicate: P, original_path: &'a Path) -> Self {
        Self {
            wrapping_predicate,
            original_path,
        }
    }
}

impl<'a, P: Predicate<str>> predicates::reflection::PredicateReflection
    for FixtureSaverPredicate<'a, P>
{
}

impl<'a, P: Predicate<str>> Predicate<str> for FixtureSaverPredicate<'a, P> {
    fn eval(&self, edit: &str) -> bool {
        if !self.wrapping_predicate.eval(edit) {
            let without_header = edit.strip_prefix(TS_FILE_HEADER).unwrap_or(edit);
            let without_header_plus_newline = format!("{without_header}\n");
            fs::write(self.original_path, without_header_plus_newline)
                .expect("failed to write fixture");
        }
        true
    }
}

impl<'a, P: Predicate<str>> std::fmt::Display for FixtureSaverPredicate<'a, P> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("FixtureSaverPredicate should never fail");
    }
}

/// Represents an instance of the qlc command
#[derive(Debug)]
pub struct TestCommandHarness {
    fixture_assert_directory: Option<PathBuf>,
    proc_cmd: Command,
    temp_dir: assert_fs::TempDir,
}

impl Default for TestCommandHarness {
    fn default() -> Self {
        Self::new_with_default_schema()
    }
}

impl TestCommandHarness {
    pub fn new() -> Self {
        // If user adds `KEEP_TEST_TEMPDIRS` arg to `cargo test`, we can keep temp_dir
        let temp_dir = assert_fs::TempDir::new()
            .expect("temp directory creation failure")
            .into_persistent_if(env::var_os("KEEP_TEST_TEMPDIRS").is_some());

        let mut proc_cmd = Command::cargo_bin("qlc").expect("qlc bin failure");
        proc_cmd.arg("--num-threads=2").arg(temp_dir.path());

        // If user adds `--nocapture` arg to `cargo test`, we can show output
        if env::args().any(|arg| arg == "--nocapture") {
            proc_cmd.stdin(Stdio::inherit());
            proc_cmd.stderr(Stdio::inherit());
        }

        Self {
            proc_cmd,
            temp_dir,
            fixture_assert_directory: None,
        }
    }

    pub fn new_with_default_schema() -> Self {
        let mut harness = Self::new();
        let default_schema_path =
            Path::new(FIXTURE_ROOT_PATH).join("schema_generation/output/schema.json");
        harness.with_default_schema_file_from_path(&default_schema_path);
        harness
    }

    pub fn directory_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn with_default_schema_file_from_path(&mut self, path: &Path) -> &mut Self {
        self.temp_dir
            .child("schema.json")
            .write_file(path)
            .expect("write schema failure");
        self
    }

    pub fn with_default_schema_file_from_contents(&mut self, contents: &str) -> &mut Self {
        self.temp_dir
            .child("schema.json")
            .write_str(contents)
            .expect("write schema failure");
        self
    }

    pub fn with_arg(&mut self, arg: impl AsRef<std::ffi::OsStr>) -> &mut Self {
        self.proc_cmd.arg(arg);
        self
    }

    pub fn with_fixture_directory(
        &mut self,
        fixture_directory_subpath: impl AsRef<Path>,
    ) -> &mut Self {
        if let Some(ref path) = self.fixture_assert_directory {
            panic!(
                "with_fixture_directory() can only be called once, already {}",
                path.display(),
            );
        }

        let subpath_ref = fixture_directory_subpath.as_ref();
        let full_fixture_directory_subpath = &Path::new(FIXTURE_ROOT_PATH).join(subpath_ref);
        self.temp_dir
            .copy_from(full_fixture_directory_subpath, &["*.graphql"])
            .expect("failure to copy graphql files from fixture dir");
        self.fixture_assert_directory = Some(subpath_ref.into());

        // Copy qlcrc but ignore errors (in case this fixture doesn't have one)
        self.add_config_file_and_arg(DEFAULT_QLCRC_JSON_PATH, |child| {
            child
                .write_file(&full_fixture_directory_subpath.join(DEFAULT_QLCRC_JSON_PATH))
                .ok()
        });

        self
    }

    pub fn with_default_rc_file_contents(&mut self, contents: &str) -> &mut Self {
        self.add_config_file_and_arg(DEFAULT_QLCRC_JSON_PATH, |child| {
            child.write_str(contents).ok()
        });
        self
    }

    pub fn run_for_failure(&mut self) -> Assert {
        let assert = self.proc_cmd.assert().failure().stderr(p_str::is_empty());
        self.assert_fixture_outputs();
        assert
    }

    pub fn run_for_success(&mut self) -> Assert {
        let assert = self.proc_cmd.assert().success().stderr(p_str::is_empty());
        self.assert_fixture_outputs();
        assert
    }

    fn add_config_file_and_arg(
        &mut self,
        child_path: impl AsRef<Path>,
        callback: impl FnOnce(&assert_fs::fixture::ChildPath) -> Option<()>,
    ) {
        let config_file_child = self.temp_dir.child(child_path);
        if callback(&config_file_child).is_some() {
            // Since this command's CWD is not the temp dir, use an argument to
            // tell qlc where it is.
            self.proc_cmd.arg("-c").arg(config_file_child.path());
        }
    }

    fn assert_fixture_outputs(&self) {
        if let Some(ref fixture_directory_subpath) = self.fixture_assert_directory {
            let full_fixture_directory_subpath =
                &Path::new(FIXTURE_ROOT_PATH).join(fixture_directory_subpath);
            let walker = globwalk::GlobWalkerBuilder::from_patterns(
                full_fixture_directory_subpath,
                &["*.ts"],
            )
            .file_type(globwalk::FileType::FILE)
            .build()
            .expect("ts glob failure");

            let should_overwrite_fixtures = env::var_os("OVERWRITE_FIXTURES").is_some();

            let ts_file_iter = walker.into_iter().filter_map(Result::ok);
            for ts_file in ts_file_iter {
                let expected_ts_file_path = ts_file.path();

                let child_path = expected_ts_file_path
                    .strip_prefix(full_fixture_directory_subpath)
                    .expect("unexpectedly not under fixtures");
                let child_path = self.temp_dir.child(child_path);

                let full_expected_file_content =
                    fs::read_to_string(expected_ts_file_path).expect("missing expected file");
                let expected_content =
                    format!("{TS_FILE_HEADER}{}", full_expected_file_content.trim());
                let diff_predicate = p_str::diff(expected_content);
                if should_overwrite_fixtures {
                    child_path.assert(FixtureSaverPredicate::new(
                        diff_predicate,
                        expected_ts_file_path,
                    ));
                } else {
                    child_path.assert(diff_predicate);
                }
            }
        }
    }
}
