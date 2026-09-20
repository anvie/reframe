use reframe::core::*;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_template_mode_default() {
    let mode = TemplateMode::default();
    assert_eq!(mode, TemplateMode::Generate);
}

fn toml_with_mode(mode: &str) -> String {
    format!(
        r#"
[reframe]
name = "Test"
author = "me"
min_version = "0.1.0"
mode = "{}"

[project]
name = "Hi"
version = "1.0"

param = []
"#,
        mode
    )
}

fn toml_without_mode() -> String {
    r#"
[reframe]
name = "Test"
author = "me"
min_version = "0.1.0"

[project]
name = "Hi"
version = "1.0"

param = []
"#
    .to_string()
}

#[test]
fn test_template_mode_deserialize_generate() {
    let config: Config = toml::from_str(&toml_with_mode("generate")).expect("should parse");
    assert_eq!(config.reframe.mode, TemplateMode::Generate);
}

#[test]
fn test_template_mode_deserialize_apply() {
    let config: Config = toml::from_str(&toml_with_mode("apply")).expect("should parse");
    assert_eq!(config.reframe.mode, TemplateMode::Apply);
}

#[test]
fn test_template_mode_deserialize_missing_defaults_to_generate() {
    let config: Config = toml::from_str(&toml_without_mode()).expect("should parse");
    assert_eq!(config.reframe.mode, TemplateMode::Generate);
}

#[test]
fn test_apply_mode_rejects_non_apply_template() {
    let tmp = std::env::temp_dir().join("reframe_test_apply_reject");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("Reframe.toml"), toml_with_mode("generate")).expect("write");

    let mut rl = rustyline::Editor::<()>::new().expect("editor");
    let result = Reframe::open(&tmp, &mut rl, false, vec![], true);
    assert!(result.is_err());
    let msg = format!("{}", result.err().unwrap());
    assert!(
        msg.contains("not in apply mode"),
        "expected apply-mode rejection, got: {}",
        msg
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_apply_mode_accepts_apply_template() {
    let tmp = std::env::temp_dir().join("reframe_test_apply_accept");
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::write(tmp.join("Reframe.toml"), toml_with_mode("apply")).expect("write");

    let mut rl = rustyline::Editor::<()>::new().expect("editor");
    let result = Reframe::open(&tmp, &mut rl, false, vec![], true);
    assert!(result.is_ok(), "apply mode should accept apply template");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_post_generate_command_deserialize() {
    let toml_str = r#"
[reframe]
name = "Test"
author = "me"
min_version = "0.1.0"

[project]
name = "Hi"
version = "1.0"

param = []

[[post_generate]]
make_executable = "_hooks/pre-commit"
command = "cp _hooks/pre-commit .git/hooks/pre-commit"
"#;
    let config: Config = toml::from_str(toml_str).expect("should parse");
    assert_eq!(config.post_generate.len(), 1);
    assert_eq!(
        config.post_generate[0].make_executable.as_deref(),
        Some("_hooks/pre-commit")
    );
    assert_eq!(
        config.post_generate[0].command.as_deref(),
        Some("cp _hooks/pre-commit .git/hooks/pre-commit")
    );
}

#[test]
fn test_post_generate_command_optional() {
    let toml_str = r#"
[reframe]
name = "Test"
author = "me"
min_version = "0.1.0"

[project]
name = "Hi"
version = "1.0"

param = []

[[post_generate]]
make_executable = "script.sh"
"#;
    let config: Config = toml::from_str(toml_str).expect("should parse");
    assert_eq!(config.post_generate.len(), 1);
    assert_eq!(
        config.post_generate[0].make_executable.as_deref(),
        Some("script.sh")
    );
    assert_eq!(config.post_generate[0].command, None);
}

#[test]
fn test_cleanup_paths_must_be_safe_relative_paths() {
    for cleanup in ["", ".", "../outside", "dir/../outside", "/tmp/outside"] {
        let toml = format!(
            "[reframe]\nname = \"Test\"\nauthor = \"me\"\nmin_version = \"0.1.0\"\ncleanup = [{:?}]\n\n[project]\nname = \"Test\"\nversion = \"1.0\"\n",
            cleanup
        );
        let root = std::env::temp_dir().join(format!(
            "reframe_cleanup_validation_{}_{}",
            std::process::id(), cleanup.len()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create template");
        std::fs::write(root.join("Reframe.toml"), toml).expect("write config");
        let mut rl = rustyline::Editor::<()>::new().expect("editor");
        assert!(Reframe::open(&root, &mut rl, false, vec![], false).is_err(), "{cleanup:?} must be rejected");
        let _ = std::fs::remove_dir_all(root);
    }
}

#[test]
fn test_apply_removes_consumed_git_staging_directory() {
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("reframe_apply_git_staging_{}_{}", std::process::id(), id));
    let template = root.join("template");
    let output = root.join("output");
    std::fs::create_dir_all(template.join("_git/info")).expect("create template");
    std::fs::create_dir_all(&output).expect("create output");
    std::fs::write(
        template.join("Reframe.toml"),
        r#"
[reframe]
name = "Apply test"
author = "test"
min_version = "0.1.0"
mode = "apply"
cleanup = ["_git"]

[project]
name = "Test"
version = "1.0"

[[post_generate]]
command = "cp _git/info/exclude consumed-exclude"
"#,
    )
    .expect("write config");
    std::fs::write(template.join("_git/info/exclude"), "*.local\n").expect("write staging file");

    let status = Command::new(env!("CARGO_BIN_EXE_reframe"))
        .args(["apply", template.to_str().expect("template path"), "--quiet"])
        .current_dir(&output)
        .status()
        .expect("run reframe");

    assert!(status.success());
    assert_eq!(
        std::fs::read_to_string(output.join("consumed-exclude")).unwrap(),
        "*.local\n\n"
    );
    assert!(!output.join("_git").exists(), "_git staging directory must be removed");

    let _ = std::fs::remove_dir_all(root);
}
