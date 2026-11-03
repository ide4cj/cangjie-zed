use std::collections::{BTreeSet, HashMap};

/// Expand overrides once against the worktree's shell environment, independent of entry order.
pub fn expand(
    extra: Vec<(String, String)>,
    inherited: &[(String, String)],
    mut warn: impl FnMut(String),
) -> Vec<(String, String)> {
    let inherited: HashMap<_, _> = inherited
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    extra.into_iter().map(|(key, value)| {
        let mut result = String::new();
        let mut missing = BTreeSet::new();
        let bytes = value.as_bytes();
        let mut offset = 0;
        let mut literal_start = 0;
        while offset < bytes.len() {
            if bytes[offset] != b'$' || offset + 1 == bytes.len() {
                offset += 1;
                continue;
            }
            let start = offset;
            let name_start = offset + 1;
            let (name, end) = if bytes[name_start] == b'{' {
                let Some(close) = value[name_start + 1..].find('}') else {
                    offset += 1;
                    continue;
                };
                let close = name_start + 1 + close;
                if close == name_start + 1 {
                    offset += 1;
                    continue;
                }
                (&value[name_start + 1..close], close + 1)
            } else {
                if !(bytes[name_start].is_ascii_alphabetic() || bytes[name_start] == b'_') {
                    offset += 1;
                    continue;
                }
                let mut end = name_start + 1;
                while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                    end += 1;
                }
                (&value[name_start..end], end)
            };
            result.push_str(&value[literal_start..start]);
            match inherited.get(name) {
                Some(replacement) => result.push_str(replacement),
                None => {
                    missing.insert(name);
                    result.push_str(&value[start..end]);
                }
            }
            offset = end;
            literal_start = end;
        }
        result.push_str(&value[literal_start..]);
        // A tilde inserted from an environment value must remain literal.
        if value == "~" || value.starts_with("~/") || value.starts_with("~\\") {
            match inherited.get("HOME").or_else(|| inherited.get("USERPROFILE")) {
                Some(home) if !home.is_empty() => result = format!("{home}{}", &result[1..]),
                _ => warn(format!("lsp.cjls.binary.env.{key}: HOME and USERPROFILE do not provide a home directory; keeping ~")),
            }
        }
        if !missing.is_empty() {
            warn(format!("lsp.cjls.binary.env.{key}: environment variables are not set: {}; keeping references", missing.into_iter().collect::<Vec<_>>().join(", ")));
        }
        (key, result)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(values: &[(&str, &str)]) -> Vec<(String, String)> {
        values
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn stdx_and_path_expand_from_the_inherited_environment() {
        let extra = pairs(&[
            (
                "STDX",
                "$HOME/.cjpm/toolchains/default/linux_x86_64_cjnative/static/stdx",
            ),
            ("PATH", "${PATH}:/tools"),
            ("EMPTY", ""),
            ("LOG", "DEBUG"),
        ]);
        let inherited = pairs(&[("HOME", "/home/user"), ("PATH", "/bin")]);
        let result = expand(extra, &inherited, |message| panic!("{message}"));
        assert_eq!(
            result,
            pairs(&[
                (
                    "STDX",
                    "/home/user/.cjpm/toolchains/default/linux_x86_64_cjnative/static/stdx"
                ),
                ("PATH", "/bin:/tools"),
                ("EMPTY", ""),
                ("LOG", "DEBUG")
            ])
        );
    }

    #[test]
    fn expansion_is_not_recursive_or_dependent_on_override_order() {
        let inherited = pairs(&[
            ("HOME", "/home/user"),
            ("SDK", "$HOME/sdk"),
            ("TILDE", "~/sdk"),
        ]);
        let extra = pairs(&[
            ("HOME", "/override"),
            ("SDK", "${SDK}"),
            ("COPY", "$HOME"),
            ("TILDE", "$TILDE"),
        ]);
        let mut reversed = extra.clone();
        reversed.reverse();
        let mut result = expand(extra, &inherited, |message| panic!("{message}"));
        let mut other = expand(reversed, &inherited, |message| panic!("{message}"));
        assert_eq!(
            result,
            pairs(&[
                ("HOME", "/override"),
                ("SDK", "$HOME/sdk"),
                ("COPY", "/home/user"),
                ("TILDE", "~/sdk")
            ])
        );
        result.sort();
        other.sort();
        assert_eq!(result, other);
    }

    #[test]
    fn home_supports_windows_and_prefers_home_over_userprofile() {
        let extra = pairs(&[
            ("ROOT", "~"),
            ("SDK", "~\\sdk"),
            ("SRC", "~/src"),
            ("LITERAL", "~user/$(pwd)"),
        ]);
        let inherited = pairs(&[("USERPROFILE", "C:\\Users\\user")]);
        assert_eq!(
            expand(extra, &inherited, |message| panic!("{message}")),
            pairs(&[
                ("ROOT", "C:\\Users\\user"),
                ("SDK", "C:\\Users\\user\\sdk"),
                ("SRC", "C:\\Users\\user/src"),
                ("LITERAL", "~user/$(pwd)")
            ])
        );
        assert_eq!(
            expand(
                pairs(&[("ROOT", "~")]),
                &pairs(&[("HOME", "/home"), ("USERPROFILE", "/fallback")]),
                |message| panic!("{message}")
            ),
            pairs(&[("ROOT", "/home")])
        );
    }

    #[test]
    fn missing_references_stay_literal_and_warnings_do_not_reveal_values() {
        let mut warnings = Vec::new();
        let result = expand(
            pairs(&[
                ("TOKEN", "secret-${MISSING}-$MISSING"),
                ("SDK", "~/sdk"),
                ("EMPTY", "$EMPTY"),
                ("UNICODE", "данные/$EMPTY/🙂"),
            ]),
            &pairs(&[("EMPTY", "")]),
            |message| warnings.push(message),
        );
        assert_eq!(
            result,
            pairs(&[
                ("TOKEN", "secret-${MISSING}-$MISSING"),
                ("SDK", "~/sdk"),
                ("EMPTY", ""),
                ("UNICODE", "данные//🙂")
            ])
        );
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("TOKEN") && warnings[0].contains("MISSING"));
        assert!(warnings[1].contains("SDK"));
        assert!(warnings.iter().all(|message| !message.contains("secret-")));
    }
}
