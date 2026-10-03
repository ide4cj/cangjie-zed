//! Cangjie in Zed: a thin client of cjls (cjls's D15, D32). Zed publishes an extension through a
//! reviewed PR, so the server is not pinned per release: the extension downloads the newest cjls
//! within the minor of the release it pins (`.cjls-version`), which it supports, and that pin
//! otherwise.

use std::fs;

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

const REPOSITORY: &str = "ide4cj/cjls";
const SERVER: &str = "cjls";
/// The release this extension is tested with; empty until cjls has one, and then the latest.
const PIN: &str = include_str!("../.cjls-version");

struct Cangjie {
    /// The binary downloaded this session and the `version` it was for, so a nightly is fetched
    /// once per session and another `version` set meanwhile is downloaded.
    downloaded: Option<(Option<String>, String)>,
}

impl zed::Extension for Cangjie {
    fn new() -> Self {
        Cangjie { downloaded: None }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let LspSettings {
            binary, settings, ..
        } = LspSettings::for_worktree(SERVER, worktree).unwrap_or_default();
        let (path, args, env) = binary.map_or((None, None, None), |b| (b.path, b.arguments, b.env));
        let args = args.unwrap_or_default();
        let env = env.map(|env| env.into_iter().collect()).unwrap_or_default();

        // `lsp.cjls.binary.path`, then `cjls` on PATH, then the download
        let command = match path.or_else(|| worktree.which(SERVER)) {
            Some(path) => path,
            None => {
                // the extension's own key among the server's settings: none reach cjls yet; once
                // `language_server_workspace_configuration` passes them (cjls's D32), without it
                let wanted = settings
                    .as_ref()
                    .and_then(|s| s.get("version"))
                    .and_then(|v| v.as_str());
                self.download(id, wanted)?
            }
        };
        Ok(zed::Command { command, args, env })
    }
}

impl Cangjie {
    /// The binary of the release `wanted` names (a tag, `nightly-build` too), else of the one this
    /// extension picks; downloaded into the extension's directory unless it is there already.
    fn download(&mut self, id: &LanguageServerId, wanted: Option<&str>) -> Result<String> {
        if let Some((was, path)) = &self.downloaded
            && was.as_deref() == wanted
            && fs::metadata(path).is_ok_and(|m| m.is_file())
        {
            return Ok(path.clone());
        }
        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let (os, arch) = zed::current_platform();
        let (target, archive) = target(os, arch).ok_or_else(|| {
            format!("there is no prebuilt cjls for {os:?} {arch:?}: build it, and set lsp.cjls.binary.path")
        })?;
        let release = self.release(wanted)?;
        let asset_name = format!("cjls-{target}.{}", archive.extension());
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("cjls {} has no {asset_name}", release.version))?;

        let dir = format!("cjls-{}", release.version);
        let exe = if matches!(os, zed::Os::Windows) {
            "cjls.exe"
        } else {
            "cjls"
        };
        let path = format!("{dir}/cjls-{target}/{exe}");
        // a nightly moves under its name: fetched again once a session
        let fresh =
            release.version != "nightly-build" && fs::metadata(&path).is_ok_and(|m| m.is_file());
        if !fresh {
            zed::set_language_server_installation_status(
                id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            fs::remove_dir_all(&dir).ok();
            zed::download_file(&asset.download_url, &dir, archive.file_type())
                .map_err(|e| format!("could not download {asset_name}: {e}"))?;
            zed::make_file_executable(&path)?;
            for entry in fs::read_dir(".").map_err(|e| e.to_string())?.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("cjls-") && name != dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }
        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::None,
        );
        self.downloaded = Some((wanted.map(str::to_owned), path.clone()));
        Ok(path)
    }

    fn release(&self, wanted: Option<&str>) -> Result<zed::GithubRelease> {
        if let Some(tag) = wanted {
            return zed::github_release_by_tag_name(REPOSITORY, tag);
        }
        let latest = zed::latest_github_release(
            REPOSITORY,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let pin = PIN.trim();
        if pin.is_empty() || same_minor(&latest.version, pin) {
            Ok(latest)
        } else {
            zed::github_release_by_tag_name(REPOSITORY, pin)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Archive {
    TarGz,
    Zip,
}

impl Archive {
    fn extension(self) -> &'static str {
        match self {
            Archive::TarGz => "tar.gz",
            Archive::Zip => "zip",
        }
    }

    fn file_type(self) -> zed::DownloadedFileType {
        match self {
            Archive::TarGz => zed::DownloadedFileType::GzipTar,
            Archive::Zip => zed::DownloadedFileType::Zip,
        }
    }
}

/// The target and archive cjls's release publishes for a platform (its release.yml), if any.
fn target(os: zed::Os, arch: zed::Architecture) -> Option<(&'static str, Archive)> {
    match (os, arch) {
        (zed::Os::Mac, zed::Architecture::Aarch64) => {
            Some(("aarch64-apple-darwin", Archive::TarGz))
        }
        (zed::Os::Linux, zed::Architecture::X8664) => {
            Some(("x86_64-unknown-linux-gnu", Archive::TarGz))
        }
        (zed::Os::Windows, zed::Architecture::X8664) => {
            Some(("x86_64-pc-windows-gnu", Archive::Zip))
        }
        _ => None,
    }
}

/// Whether two tags `vMAJOR.MINOR.PATCH` share their major and minor: a 0.x minor may break.
fn same_minor(a: &str, b: &str) -> bool {
    fn minor(tag: &str) -> Option<(&str, &str)> {
        let mut parts = tag.strip_prefix('v')?.split('.');
        Some((parts.next()?, parts.next()?))
    }
    minor(a).is_some_and(|m| Some(m) == minor(b))
}

zed::register_extension!(Cangjie);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_release_is_within_the_minor_of_the_pin() {
        assert!(same_minor("v0.1.3", "v0.1.0"));
    }

    #[test]
    fn a_new_minor_is_not() {
        assert!(!same_minor("v0.2.0", "v0.1.0"));
        assert!(!same_minor("v1.1.0", "v0.1.0"));
    }

    #[test]
    fn a_tag_that_is_no_version_is_never_within_a_minor() {
        assert!(!same_minor("nightly", "v0.1.0"));
        assert!(!same_minor("v0.1.0", ""));
    }

    #[test]
    fn the_three_platforms_of_a_release_have_a_target_and_others_none() {
        assert_eq!(
            target(zed::Os::Mac, zed::Architecture::Aarch64),
            Some(("aarch64-apple-darwin", Archive::TarGz))
        );
        assert_eq!(
            target(zed::Os::Windows, zed::Architecture::X8664),
            Some(("x86_64-pc-windows-gnu", Archive::Zip))
        );
        assert_eq!(target(zed::Os::Mac, zed::Architecture::X8664), None);
    }
}
