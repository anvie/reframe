use reframe::core::*;

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
