use crate::helpers::cmd::TestCommandHarness;

#[test]
fn compile_custom_scalar_any() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/field/compile_custom_scalar_any")
        .run_for_success();
}

#[test]
fn compile_custom_scalar_with_default_names() {
    TestCommandHarness::default()
        .with_arg("--use-custom-scalars")
        .with_fixture_directory("typescript/field/compile_custom_scalar_with_default_names")
        .run_for_success();
}

#[test]
fn compile_custom_scalar_with_prefixed_names() {
    TestCommandHarness::default()
        .with_arg("--use-custom-scalars")
        .with_arg("--custom-scalar-prefix=Prefix")
        .with_fixture_directory("typescript/field/compile_custom_scalar_with_prefixed_names")
        .run_for_success();
}

#[test]
fn compile_fields_with_deprecation_marker() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/field/compile_fields_with_deprecation_marker")
        .run_for_success();
}

#[test]
fn compile_fields_without_readonly_marker() {
    TestCommandHarness::default()
        .with_arg("--disable-readonly-types")
        .with_fixture_directory("typescript/field/compile_fields_without_readonly_marker")
        .run_for_success();
}
