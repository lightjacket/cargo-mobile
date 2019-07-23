use crate::{
    config::Config,
    util::{self, IntoResult},
};
use std::{path::Path, process::Command};

#[derive(Debug, derive_more::From)]
pub enum ProjectCreationError {
    TemplateProcessingError(bicycle::ProcessingError),
    SymlinkRustError(util::CommandError),
    SymlinkResourcesError(util::CommandError),
}

pub fn create(config: &Config, bike: &bicycle::Bicycle) -> Result<(), ProjectCreationError> {
    let src = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/xcode_project"
    ));
    let dest = config.ios().project_root();
    bike.process(src, &dest, |map| config.insert_template_data(map))?;

    util::relative_symlink(config.source_root(), &dest)
        .map_err(ProjectCreationError::SymlinkRustError)?;
    util::relative_symlink(config.asset_path(), &dest)
        .map_err(ProjectCreationError::SymlinkResourcesError)?;

    Command::new("chmod")
        .arg("+x")
        .arg(dest.join("cargo_xcode.sh"))
        .status()
        .into_result()
        .expect("Failed to run `chmod`");
    // Note that Xcode doesn't always reload the project nicely; reopening is
    // often necessary.
    Command::new("xcodegen")
        .args(&["generate", "--spec"])
        .arg(dest.join("project.yml"))
        .status()
        .into_result()
        .expect("Failed to run `xcodegen`");
    Ok(())
}