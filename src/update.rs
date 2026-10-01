use crate::install;

pub fn update() -> Result<(), String> {
    install::install()
}
