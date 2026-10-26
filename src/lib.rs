//! Cangjie in Zed: a thin client of cjls (cjls's D15, D32). Zed publishes an extension through a
//! reviewed PR, so the server is not pinned per release: the extension downloads the newest cjls
//! within the minor of the release it pins (`.cjls-version`), which it supports, and that pin
//! otherwise.

use std::fs;

use zed_extension_api::{
    self as zed, LanguageServerId, Result,
    serde_json::{Value, json},
    settings::LspSettings,
};

const REPOSITORY: &str = "ide4cj/cjls";
const SERVER: &str = "cjls";
/// The section of the client's configuration the server reads its settings from (cjls's D79).
const SECTION: &str = "cjls";
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
                // the extension's own key among the server's settings, which cjls leaves alone
                let wanted = settings
                    .as_ref()
                    .and_then(|s| s.get("version"))
                    .and_then(|v| v.as_str());
                self.download(id, wanted)?
            }
        };
        Ok(zed::Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        _: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<Value>> {
        Ok(initialization_options(settings(worktree)))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<Value>> {
        Ok(workspace_configuration(settings(worktree)))
    }
}

/// `lsp.cjls.settings` in Zed's settings: the server's (cjls's D79), and the extension's `version`.
fn settings(worktree: &zed::Worktree) -> Option<Value> {
    LspSettings::for_worktree(SERVER, worktree)
        .ok()
        .and_then(|s| s.settings)
}

/// What the server starts with: its settings, over which Zed merges `lsp.cjls.initialization_options`.
fn initialization_options(settings: Option<Value>) -> Option<Value> {
    settings
}

/// What Zed answers the server's `workspace/configuration` from, the section `cjls` of it, and sends in
/// `workspace/didChangeConfiguration` when the settings change. Without settings Zed answers `null`
/// for the section, which leaves the server's as they are: `initialization_options` alone, if set.
fn workspace_configuration(settings: Option<Value>) -> Option<Value> {
    settings.map(|settings| json!({ SECTION: settings }))
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
    fn the_settings_are_what_the_server_starts_with_and_its_section_cjls() {
        // arrange
        let settings = json!({ "linkedProjects": ["/linked"], "version": "nightly-build" });

        // act
        let options = initialization_options(Some(settings.clone()));
        let configuration = workspace_configuration(Some(settings.clone()));

        // assert
        assert_eq!(options, Some(settings.clone()));
        assert_eq!(configuration, Some(json!({ "cjls": settings })));
    }

    #[test]
    fn without_settings_there_is_no_section_and_no_options() {
        // arrange
        let settings = None;

        // act
        let options = initialization_options(settings.clone());
        let configuration = workspace_configuration(settings);

        // assert
        assert_eq!(options, None);
        assert_eq!(configuration, None);
    }

    #[test]
    fn a_patch_release_is_within_the_minor_of_the_pin() {
        // arrange
        let release = "v0.1.3";
        let pin = "v0.1.0";

        // act
        let within_minor = same_minor(release, pin);

        // assert
        assert!(within_minor);
    }

    #[test]
    fn a_new_minor_is_not() {
        // arrange
        let pin = "v0.1.0";

        // act
        let newer_minor = same_minor("v0.2.0", pin);
        let newer_major = same_minor("v1.1.0", pin);

        // assert
        assert!(!newer_minor);
        assert!(!newer_major);
    }

    #[test]
    fn a_tag_that_is_no_version_is_never_within_a_minor() {
        // arrange
        let release = "v0.1.0";

        // act
        let nightly = same_minor("nightly", release);
        let empty_pin = same_minor(release, "");

        // assert
        assert!(!nightly);
        assert!(!empty_pin);
    }

    #[test]
    fn the_three_platforms_of_a_release_have_a_target_and_others_none() {
        // arrange
        let mac_arm = (zed::Os::Mac, zed::Architecture::Aarch64);
        let linux_x64 = (zed::Os::Linux, zed::Architecture::X8664);
        let windows_x64 = (zed::Os::Windows, zed::Architecture::X8664);
        let mac_x64 = (zed::Os::Mac, zed::Architecture::X8664);

        // act
        let mac = target(mac_arm.0, mac_arm.1);
        let linux = target(linux_x64.0, linux_x64.1);
        let windows = target(windows_x64.0, windows_x64.1);
        let unsupported = target(mac_x64.0, mac_x64.1);

        // assert
        assert_eq!(mac, Some(("aarch64-apple-darwin", Archive::TarGz)));
        assert_eq!(linux, Some(("x86_64-unknown-linux-gnu", Archive::TarGz)));
        assert_eq!(windows, Some(("x86_64-pc-windows-gnu", Archive::Zip)));
        assert_eq!(unsupported, None);
    }

    /// Zed ignores a key it does not know: a misspelled `token_modifiers` would make the rule match
    /// every token of its type.
    #[test]
    fn every_semantic_token_rule_names_a_type_and_a_style_in_keys_zed_reads() {
        // arrange
        const KEYS: [&str; 9] = [
            "token_type",
            "token_modifiers",
            "style",
            "foreground_color",
            "background_color",
            "underline",
            "strikethrough",
            "font_weight",
            "font_style",
        ];
        let source = include_str!("../languages/cangjie/semantic_token_rules.json");

        // act
        let rules: Value =
            zed::serde_json::from_str(source).expect("semantic_token_rules.json is JSON");
        let rules = rules.as_array().expect("an array of rules");

        // assert
        assert!(!rules.is_empty());
        for rule in rules {
            let rule = rule.as_object().expect("a rule is an object");
            assert!(
                rule.keys().all(|key| KEYS.contains(&key.as_str())),
                "{rule:?}"
            );
            assert!(rule["token_type"].is_string(), "{rule:?}");
            assert!(
                rule["style"]
                    .as_array()
                    .is_some_and(|style| !style.is_empty()),
                "{rule:?}"
            );
        }
    }
}
