use std::{
    env,
    io::{self, ErrorKind},
    path::PathBuf,
};

pub fn local_app_data_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_app_data = env::var("LOCALAPPDATA").unwrap_or_default();
    if local_app_data.is_empty() {
        return Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            "LOCALAPPDATA environment variable not set",
        )));
    }
    Ok(PathBuf::from(local_app_data).join("ntix"))
}