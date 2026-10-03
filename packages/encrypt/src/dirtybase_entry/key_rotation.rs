use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use dirtybase_contract::config_contract::DirtyConfig;

const KEY: &str = "DTY_APP_KEY";
const PREVIOUS: &str = "DTY_APP_PREVIOUS_KEYS";

#[derive(Default, serde::Deserialize)]
pub(super) struct ConfiguredKeys {
    #[serde(default)]
    key: String,
    #[serde(default, deserialize_with = "previous_keys")]
    previous_keys: String,
}

fn previous_keys<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    // The app's TOML template uses an array, while dotenv uses a comma-separated string.
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Keys {
        Text(String),
        List(Vec<String>),
    }
    Ok(
        match <Keys as serde::Deserialize>::deserialize(deserializer)? {
            Keys::Text(value) => value,
            Keys::List(values) => values.join(","),
        },
    )
}

impl ConfiguredKeys {
    pub(super) async fn load(config: &DirtyConfig) -> anyhow::Result<Self> {
        Ok(config
            .load_optional_file_fn("app.toml", Some("DTY_APP"), |env| env)
            .build()
            .await?
            .try_deserialize()?)
    }
}

fn read_env(path: &Path) -> anyhow::Result<String> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error).with_context(|| format!("could not read {}", path.display())),
    }
}

fn values(contents: &str) -> anyhow::Result<HashMap<String, String>> {
    // Match dotenv override loading: the last assignment wins.
    dotenvy::from_read_iter(contents.as_bytes())
        .collect::<Result<_, _>>()
        .map_err(|_| anyhow::anyhow!("invalid dotenv file"))
}

pub(super) fn write_key(
    config: &DirtyConfig,
    keys: &ConfiguredKeys,
    new_key: &str,
) -> anyhow::Result<PathBuf> {
    let dir = config.dotenv_dir();
    let dev_path = dir.join(".env.dev");
    let dev = read_env(&dev_path)?;
    let dev_values = values(&dev)?;
    // .env.dev is loaded after .env, so rotate there when it overrides either key.
    let path = if dev_values.contains_key(KEY) || dev_values.contains_key(PREVIOUS) {
        dev_path
    } else {
        dir.join(".env")
    };
    let existing = read_env(&path)?;
    let entries = values(&existing)?;
    let mut previous = Vec::new();
    for value in [
        Some(keys.key.as_str()),
        entries.get(KEY).map(String::as_str),
        Some(keys.previous_keys.as_str()),
        entries.get(PREVIOUS).map(String::as_str),
    ]
    .into_iter()
    .flatten()
    {
        for key in value
            .split(',')
            .map(str::trim)
            .filter(|key| !key.is_empty() && *key != new_key)
        {
            if !previous.contains(&key) {
                previous.push(key);
            }
        }
    }
    // The replacement below operates on physical lines. Reject multiline
    // assignments before touching the file rather than risk removing only part
    // of an assignment (or key-like text inside an unrelated value).
    for line in existing.lines() {
        values(line).context("cannot rotate keys in a dotenv file with multiline assignments")?;
    }
    let mut lines = existing.lines().map(str::to_owned).collect::<Vec<_>>();
    set_value(&mut lines, KEY, new_key);
    set_value(&mut lines, PREVIOUS, &previous.join(","));
    let output = format!("{}\n", lines.join("\n"));
    atomic_write(&path, |file| file.write_all(output.as_bytes()))?;
    Ok(path)
}

fn set_value(lines: &mut Vec<String>, key: &str, value: &str) {
    // Remove every assignment so a later duplicate cannot undo the rotation.
    lines.retain(|line| {
        let line = line.trim();
        let line = line
            .strip_prefix("export")
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .unwrap_or(line)
            .trim_start();
        !line
            .split_once('=')
            .is_some_and(|(name, _)| name.trim() == key)
    });
    lines.push(format!("{key}=\"{value}\""));
}

fn atomic_write(
    path: &Path,
    write: impl FnOnce(&mut fs::File) -> std::io::Result<()>,
) -> anyhow::Result<()> {
    // Write through symlinks so a shared `.env` is updated rather than replaced.
    let resolved = match fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => path.to_path_buf(),
        Err(error) => return Err(error.into()),
    };
    let path = resolved.as_path();
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    if let Ok(metadata) = fs::metadata(path) {
        temp.as_file().set_permissions(metadata.permissions())?;
    }
    write(temp.as_file_mut())?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Encrypter;
    use base64ct::Encoding;

    fn config_at(dir: &Path) -> DirtyConfig {
        // Deserialize rather than loading files into the process environment.
        serde_json::from_value(serde_json::json!({
            "app_name": "test", "current_env": "dev", "config_dir": "unused",
            "dotenv_dir": dir
        }))
        .unwrap()
    }

    #[test]
    fn rotation_preserves_effective_keys_and_decrypts_old_data() {
        let dir = tempfile::tempdir().unwrap();
        let old = Encrypter::generate_aes256gcm_key();
        let older = Encrypter::generate_aes256gcm_key();
        let new = Encrypter::generate_aes256gcm_key();
        let keys = ConfiguredKeys {
            key: Encrypter::key_to_env_value(&old),
            previous_keys: Encrypter::key_to_env_value(&older),
        };
        let path = write_key(
            &config_at(dir.path()),
            &keys,
            &Encrypter::key_to_env_value(&new),
        )
        .unwrap();
        assert_eq!(path, dir.path().join(".env"));
        let values = values(&fs::read_to_string(path).unwrap()).unwrap();
        let previous = values[PREVIOUS]
            .split(',')
            .map(|key| base64ct::Base64::decode_vec(key.strip_prefix("base64:").unwrap()).unwrap())
            .collect();
        let rotated = Encrypter::new(&new, Some(previous));
        for key in [old, older] {
            let ciphertext = Encrypter::new(&key, None)
                .encrypt_str("still readable")
                .unwrap();
            assert_eq!(rotated.decrypt(&ciphertext).unwrap(), b"still readable");
        }
    }

    #[test]
    fn updates_highest_priority_file_and_removes_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env.dev");
        fs::write(&path, "# keep\nOTHER=yes\nDTY_APP_KEY=first\nexport DTY_APP_KEY='last' # comment\nDTY_APP_PREVIOUS_KEYS=older\n").unwrap();
        let keys = ConfiguredKeys {
            key: "last".into(),
            previous_keys: "older".into(),
        };
        assert_eq!(
            write_key(&config_at(dir.path()), &keys, "new").unwrap(),
            path
        );
        let result = fs::read_to_string(path).unwrap();
        assert!(result.starts_with("# keep\nOTHER=yes\n"));
        assert_eq!(result.matches("DTY_APP_KEY=").count(), 1);
        let entries = values(&result).unwrap();
        assert_eq!(entries[KEY], "new");
        assert_eq!(entries[PREVIOUS], "last,older");
        assert!(!dir.path().join(".env").exists());
    }

    #[test]
    fn failed_write_preserves_original_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        fs::write(&path, "DTY_APP_KEY=original\n").unwrap();
        let result = atomic_write(&path, |file| {
            file.write_all(b"partial")?;
            Err(std::io::Error::other("simulated write failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "DTY_APP_KEY=original\n");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_updates_symlink_target() {
        let dir = tempfile::tempdir().unwrap();
        let shared = dir.path().join("shared.env");
        let link = dir.path().join(".env");
        fs::write(&shared, "DTY_APP_KEY=original\n").unwrap();
        std::os::unix::fs::symlink(&shared, &link).unwrap();

        atomic_write(&link, |file| file.write_all(b"DTY_APP_KEY=rotated\n")).unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_to_string(&shared).unwrap(),
            "DTY_APP_KEY=rotated\n"
        );
    }

    #[test]
    fn rejects_multiline_assignments_without_changing_file() {
        for contents in [
            "DTY_APP_PREVIOUS_KEYS=\"base64:first,\nbase64:second\"\n",
            "OTHER=\"first line\nDTY_APP_KEY=not-an-assignment\nlast line\"\n",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join(".env");
            // These are valid dotenv files, but cannot safely be edited by
            // the physical-line replacement used by key rotation.
            assert!(values(contents).is_ok());
            fs::write(&path, contents).unwrap();
            let error =
                write_key(&config_at(dir.path()), &ConfiguredKeys::default(), "new").unwrap_err();
            assert!(error.to_string().contains("multiline assignments"));
            assert_eq!(fs::read_to_string(&path).unwrap(), contents);
            assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        }
    }

    #[test]
    fn accepts_empty_toml_previous_keys() {
        let config = dirtybase_contract::config_contract::config::Config::builder()
            .add_source(dirtybase_contract::config_contract::config::File::from_str(
                "key = 'old'\nprevious_keys = []",
                dirtybase_contract::config_contract::config::FileFormat::Toml,
            ))
            .build()
            .unwrap();
        let keys: ConfiguredKeys = config.try_deserialize().unwrap();
        assert_eq!(keys.key, "old");
        assert!(keys.previous_keys.is_empty());
    }
}
