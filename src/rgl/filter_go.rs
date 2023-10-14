use super::{Filter, RglError, RglResult, Subprocess};
use serde::{Deserialize, Serialize};
use simplelog::info;
use simplelog::warn;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct FilterGo {
    pub name: String,
    pub script: String,
}

impl FilterGo {
    pub fn new(name: &str, script: &str) -> Self {
        Self {
            name: name.to_owned(),
            script: script.to_owned(),
        }
    }
}

impl Filter for FilterGo {
    fn install_dependencies(&self, filter_dir: PathBuf) -> RglResult<()> {
        info!("Installing go dependencies for <b>{}</>...", self.name);
        let go = "go";
        Subprocess::new(go)
            .args(vec!["mod", "download"])
            .current_dir(filter_dir)
            .run_silent()?;
        Ok(())
    }
    fn run(&mut self, temp: &PathBuf, run_args: &Vec<String>) -> RglResult<()> {
        let script = match Path::new(&self.script).canonicalize() {
            Ok(script) => script.display().to_string(),
            Err(_) => {
                return Err(RglError::InvalidFilterDefinition {
                    filter_name: self.name.to_owned(),
                    cause: RglError::PathNotExists {
                        path: self.script.to_owned(),
                    }
                    .into(),
                })
            }
        };

        let script_dir = Path::new(&script).parent().unwrap();
        const FILES: [&str; 2] = ["go.mod", "go.sum"];
        for file in FILES.iter() {
            let f = script_dir.join(file);
            if let Err(_) = std::fs::copy(&f, &temp.join(f.file_name().unwrap())) {
                warn!("Failed to copy <b>{}</>", file);
            }
        }

        let output = Subprocess::new("go")
            .arg("run")
            .arg(&script)
            .args(run_args)
            .current_dir(temp)
            .setup_env()?
            .run()?;

        match output.status.success() {
            true => Ok(()),
            false => Err(RglError::FilterRunFailed {
                filter_name: self.name.to_owned(),
            }),
        }
    }
}
