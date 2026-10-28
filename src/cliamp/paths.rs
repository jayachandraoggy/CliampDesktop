use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "cliamp")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("cliamp")
        })
}

pub fn socket_path() -> PathBuf {
    config_dir().join("cliamp.sock")
}
