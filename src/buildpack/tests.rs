use super::{Build, BuildPack};
use crate::buildpack::{BuildPackProject, Env};
use std::{fs, vec};
use tempfile::NamedTempFile;

#[test]
fn parse_test() {
    let ref_toml = r#"
[project]
id = "io.buildpacks.my-app"
version = "0.1"

[build]
include = [
"cmd/",
"go.mod",
"go.sum",
"*.go"
]

[[build.env]]
name = "JAVA_OPTS"
value = "-Xmx1g"

[[build.buildpacks]]
id = "io.buildpacks/java"
version = "1.0"

[[build.buildpacks]]
id = "io.buildpacks/nodejs"
version = "1.0"

[metadata]
foo = "bar"

[metadata.fizz]
buzz = ["a", "b", "c"]

"#;
    let toml: BuildPackProject = toml::from_str(ref_toml).unwrap();
    let toml2: BuildPackProject = toml::from_str(&toml::to_string(&toml).unwrap()).unwrap();

    assert_eq!(toml, toml2);
}

#[test]
fn field_test() {
    let bp = BuildPackProject {
        project: None,
        metadata: None,
        build: Some(Build {
            env: Some(vec![Env {
                name: None,
                value: None,
            }]),
            file_list: None,
            buildpacks: Some(vec![BuildPack {
                id: None,
                buildpack_field: super::BuildPackField::Version(None),
            }]),
        }),
    };

    let temp = NamedTempFile::new().expect("failed creating a new temp file");

    let temp = temp.path();

    bp.export_toml(temp).expect("failed exporting toml");

    let str = fs::read_to_string(temp).expect("failed to read toml");

    let ret: BuildPackProject = toml::from_str(&str).expect("failed to deserialize");

    assert_eq!(bp, ret);
}
