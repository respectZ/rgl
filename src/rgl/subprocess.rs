use super::{RglError, RglResult};
use dunce::canonicalize;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process,
};

pub struct Subprocess {
    command: process::Command,
}

impl Subprocess {
    pub fn new<S>(command: S) -> Self
    where
        S: AsRef<OsStr>,
    {
        Self {
            command: process::Command::new(command),
        }
    }

    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<OsStr>,
    {
        self.command.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn current_dir<P>(&mut self, dir: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.command.current_dir(dir);
        self
    }

    pub fn setup_env(&mut self, filter_dir: &PathBuf) -> RglResult<&mut Self> {
        let root_dir = canonicalize(".").map_err(|err| RglError::Wrap(err.into()))?;
        let filter_dir = canonicalize(filter_dir).map_err(|err| RglError::Wrap(err.into()))?;
        self.command
            .env("ROOT_DIR", root_dir)
            .env("FILTER_DIR", filter_dir);
        Ok(self)
    }

    pub fn run(&mut self) -> RglResult<process::Output> {
        match self.command.spawn() {
            Ok(handler) => match handler.wait_with_output() {
                Ok(output) => Ok(output),
                Err(e) => Err(RglError::Subprocess {
                    cause: RglError::Wrap(e.into()).into(),
                }),
            },
            Err(e) => Err(RglError::Subprocess {
                cause: RglError::Wrap(e.into()).into(),
            }),
        }
    }

    pub fn run_silent(&mut self) -> RglResult<process::Output> {
        match self.command.output() {
            Ok(output) => Ok(output),
            Err(e) => Err(RglError::Subprocess {
                cause: RglError::Wrap(e.into()).into(),
            }),
        }
    }
}
