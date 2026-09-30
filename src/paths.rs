use std::path::PathBuf;

pub fn kome_home() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("KOME_HOME") {
        return Ok(PathBuf::from(path));
    }

    let home =
        dirs::home_dir().ok_or_else(|| "home directory could not be determined".to_string())?;

    Ok(home.join(".kome"))
}
