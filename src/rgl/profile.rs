use super::{find_mojang_dir, RunContext};
use crate::{info, measure_time};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub export: Export,
    pub filters: Vec<FilterRunner>,
}

#[derive(Serialize, Deserialize)]
pub struct Export {
    pub target: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterRunner {
    Filter {
        #[serde(rename = "filter")]
        filter_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        arguments: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<HashMap<String, Value>>,
    },
    ProfileFilter {
        #[serde(rename = "profile")]
        profile_name: String,
    },
}

impl Profile {
    pub fn run(&self, context: &RunContext, temp: &PathBuf) -> Result<()> {
        for entry in self.filters.iter() {
            match entry {
                FilterRunner::Filter {
                    filter_name,
                    arguments,
                    settings,
                } => {
                    let filter_def = context.get_filter_def(filter_name)?;
                    let mut run_args: Vec<String> = vec![];
                    if let Some(settings) = settings {
                        run_args = vec![serde_json::to_string(settings).unwrap()]
                    }
                    if let Some(args) = arguments {
                        run_args.extend(args.iter().map(|x| x.to_owned()));
                    }

                    measure_time!(filter_name, {
                        info!("Running filter <b>{filter_name}</>");
                        filter_def
                            .to_filter(filter_name, None)?
                            .run(temp, &run_args)
                            .context(format!("Failed running filter <b>{filter_name}</>"))?;
                    });
                }
                FilterRunner::ProfileFilter { profile_name } => {
                    if profile_name == &context.root_profile {
                        bail!("Found circular profile reference in <b>{profile_name}</>");
                    }
                    let profile = context.get_profile(profile_name)?;

                    info!("Running <b>{profile_name}</> nested profile");
                    profile.run(context, temp)?;
                }
            }
        }
        Ok(())
    }

    pub fn get_export_paths(&self, name: &str) -> Result<(PathBuf, PathBuf)> {
        let target = self.export.target.as_str();
        match target {
            "development" => {
                let mojang_dir = find_mojang_dir()?;
                let bp = mojang_dir
                    .join("development_behavior_packs")
                    .join(format!("{}_bp", name));
                let rp = mojang_dir
                    .join("development_resource_packs")
                    .join(format!("{}_rp", name));
                Ok((bp, rp))
            }
            "local" => {
                let build = PathBuf::from("build");
                if !build.exists() {
                    fs::create_dir(&build)?;
                }
                let bp = build.join("BP");
                let rp = build.join("RP");
                Ok((bp, rp))
            }
            _ => bail!("Export target <b>{target}</> is not valid"),
        }
    }

    pub fn get_temp_dir(&self) -> Result<PathBuf> {
        let target = self.export.target.as_str();
        match target {
            "development" => Ok(find_mojang_dir()?.join(".regolith")),
            _ => Ok(PathBuf::from(".regolith").join("tmp")),
        }
    }
}
