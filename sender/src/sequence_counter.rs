use anyhow::{anyhow, Context, Error, Result};
use std::{fs, path::PathBuf};

pub struct SequenceCounter {
    path: PathBuf,
    value: u32,
}

impl SequenceCounter {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();

        if !path.exists() {
            fs::write(&path, "0\n")
                .with_context(|| format!("failed to create sequence counter {}", path.display()))?;
        }

        restrict_owner_access(&path)?;

        let data = fs::read_to_string(&path)
            .with_context(|| format!("failed to read sequence counter {}", path.display()))?;
        let value = match data.trim() {
            "" => 0,
            raw => raw
                .parse::<u32>()
                .with_context(|| format!("invalid sequence counter value {raw:?}"))?,
        };

        Ok(Self { path, value })
    }

    pub fn current(&self) -> u32 {
        self.value
    }

    pub fn increment(&mut self) -> Result<(), Error> {
        self.value = self
            .value
            .checked_add(1)
            .ok_or_else(|| anyhow!("sequence counter overflowed"))?;
        fs::write(&self.path, format!("{}\n", self.value)).with_context(|| {
            format!("failed to update sequence counter {}", self.path.display())
        })?;
        restrict_owner_access(&self.path)?;
        Ok(())
    }
}

#[cfg(unix)]
fn restrict_owner_access(path: &PathBuf) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to restrict permissions for {}", path.display()))
}

#[cfg(not(unix))]
fn restrict_owner_access(_path: &PathBuf) -> Result<(), Error> {
    Ok(())
}
