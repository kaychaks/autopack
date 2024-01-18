mod serde_macro;

use crate::ser_deser_str_with_def;
use serde::{Deserialize, Serialize};
use toml::value::Table;

pub(crate) fn default_version() -> String {
    "latest".to_string()
}

pub(crate) fn default_uri() -> String {
    "urn::buildpack::<id>".to_string()
}

pub(crate) fn default_shell() -> String {
    "/bin/sh".to_string()
}

ser_deser_str_with_def!(VersionSerDeser, default_version);
ser_deser_str_with_def!(UriSerDeser, default_uri);
ser_deser_str_with_def!(ShellSerDeser, default_shell);

/// Specification following the BuildPack project descriptor
/// https://buildpacks.io/docs/reference/config/project-descriptor/
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BuildPackProject {
    pub(super) project: Option<Project>,
    pub(super) build: Option<Build>,
    pub(super) metadata: Option<Metadata>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Project {
    pub(super) id: Option<String>,
    pub(super) name: Option<String>,
    pub(super) version: Option<String>,
    pub(super) source_url: Option<String>,
    pub(super) documentation_url: Option<String>,
    pub(super) authors: Option<Vec<String>>,
    pub(super) licenses: Option<Vec<License>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(super) struct Build {
    #[serde(flatten)]
    pub(super) file_list: Option<FileList>,
    pub(super) env: Option<Vec<Env>>,
    pub(super) buildpacks: Option<Vec<BuildPack>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub(super) enum Metadata {
    #[serde(serialize_with = "toml::ser::tables_last")]
    Meta(Table),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(super) struct License {
    #[serde(rename = "type")]
    pub(super) licence_type: Option<String>,
    pub(super) uri: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(super) struct BuildPack {
    pub(super) id: Option<String>,
    #[serde(flatten)]
    pub(super) buildpack_field: BuildPackField,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(super) enum BuildPackField {
    #[serde(rename = "version", with = "VersionSerDeser")]
    Version(Option<String>),
    #[serde(rename = "uri", with = "UriSerDeser")]
    Uri(Option<String>),
    #[serde(rename = "script")]
    Script(Option<Vec<Script>>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(super) struct Script {
    pub(super) api: String,
    pub(super) inline: String,
    #[serde(with = "ShellSerDeser")]
    pub(super) shell: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(super) struct Env {
    pub(super) name: Option<String>,
    pub(super) value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub(super) enum FileList {
    Include(Vec<String>),
    Exclude(Vec<String>),
}
