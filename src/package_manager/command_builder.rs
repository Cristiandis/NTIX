use std::{io, sync::LazyLock};

use regex::Regex;

use crate::models::options::{ChocoOptions, ScoopOptions};

static SAFE_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._\-/]+$").unwrap());

pub fn validate_id(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    if id.trim().is_empty() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "Package ID cannot be empty").into(),
        );
    }

    if !SAFE_ID_PATTERN.is_match(id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Package ID contains invalid characters: {id}"),
        )
        .into());
    }

    Ok(())
}

pub fn build_choco_install(
    id: &str,
    version: Option<&str>,
    opts: ChocoOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    let mut cmd = format!("choco install {id}");
    if let Some(v) = version {
        validate_id(v)?;
        cmd += format!(" --version {v}").as_str();
    }
    if opts.yes {
        cmd += " -y";
    }
    if opts.force {
        cmd += " --force";
    }
    if opts.ignore_dependencies {
        cmd += " --ignore-dependencies";
    }
    if opts.allow_downgrade {
        cmd += " --allow-downgrade";
    }
    if opts.skip_power_shell {
        cmd += " --skip-scripts";
    }
    if opts.pre {
        cmd += " --pre";
    }
    if let Some(p) = opts.params {
        cmd += format!(" --params=\"'{}'\"", p).as_str();
    }

    Ok(cmd)
}

pub fn build_scoop_install(
    id: &str,
    version: Option<&str>,
    opts: ScoopOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    let mut cmd = if let Some(v) = version {
        validate_id(v)?;
        format!("scoop install {id}@{v}")
    } else {
        format!("scoop install {id}")
    };
    if opts.global {
        cmd += " -g";
    };
    if opts.independent {
        cmd += " -i";
    };
    if opts.no_cache {
        cmd += " -k";
    };
    if opts.skip_hash_check {
        cmd += " -s";
    };
    if let Some(a) = opts.arch {
        cmd += format!(" --arch {}", a).as_str()
    };
    Ok(cmd)
}

pub fn build_choco_upgrade(
    id: &str,
    opts: ChocoOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    let mut cmd = format!("choco upgrade {id}");
    if opts.yes {
        cmd += " -y";
    };
    if opts.force {
        cmd += " --force";
    };
    if opts.pre {
        cmd += " --pre";
    };
    Ok(cmd)
}

pub fn build_scoop_upgrade(
    id: &str,
    opts: ScoopOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    let mut cmd = format!("scoop update {id}");
    if opts.global {
        cmd += " -g";
    };
    Ok(cmd)
}

pub fn build_choco_uninstall(
    id: &str,
    opts: ChocoOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    let mut cmd = format!("choco uninstall {id}");
    if opts.yes {
        cmd += " -y";
    };
    if opts.force {
        cmd += " --force";
    };
    if opts.ignore_dependencies {
        cmd += " --ignore-dependencies";
    };
    Ok(cmd)
}

pub fn build_scoop_uninstall(
    id: &str,
    opts: ScoopOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    let mut cmd = format!("scoop uninstall {id}");
    if opts.global {
        cmd += " -g";
    };
    Ok(cmd)
}

pub fn build_choco_search(id: &str) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    Ok(format!("choco search {id} --limit-output"))
}

pub fn build_scoop_info(id: &str) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(id)?;
    Ok(format!("scoop info {id}"))
}

pub fn build_scoop_bucket_add(
    name: &str,
    url: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(name)?;
    if let Some(u) = url {
        Ok(format!("scoop bucket add {name} {u}"))
    } else {
        Ok(format!("scoop bucket add {name}"))
    }
}

pub fn build_scoop_bucket_list() -> String {
    "scoop bucket list".to_string()
}

pub fn build_scoop_bucket_remove(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    validate_id(name)?;
    Ok(format!("scoop bucket rm {name}"))
}
