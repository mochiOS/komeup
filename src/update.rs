use crate::install;

pub fn update(version: &str) -> Result<(), String> {
    install::install(version)
}
