use std::collections::{BTreeSet, HashMap, HashSet};

/// Resolve references between overrides, using inherited values literally as fallback.
pub fn expand(
    extra: Vec<(String, String)>,
    inherited: &[(String, String)],
    mut warn: impl FnMut(String),
) -> Vec<(String, String)> {
    let mut resolver = Resolver {
        extra: extra.iter().cloned().collect(),
        inherited: inherited.iter().cloned().collect(),
        resolved: HashMap::new(),
        active: HashSet::new(),
        warnings: Vec::new(),
    };
    let result = extra
        .into_iter()
        .map(|(key, original)| {
            let value = resolver.resolve(&key).unwrap_or(original);
            (key, value)
        })
        .collect();
    for message in resolver.warnings {
        warn(message);
    }
    result
}

struct Resolver {
    extra: HashMap<String, String>,
    inherited: HashMap<String, String>,
    // None memoizes cycle failures so dependents also preserve their original values.
    resolved: HashMap<String, Option<String>>,
    active: HashSet<String>,
    warnings: Vec<String>,
}

impl Resolver {
    fn resolve(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.resolved.get(key) {
            return value.clone();
        }
        if !self.active.insert(key.to_string()) {
            return None;
        }
        let value = self.extra[key].clone();
        let mut result = String::new();
        let mut missing = BTreeSet::new();
        let mut valid = true;
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
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                (&value[name_start..end], end)
            };
            result.push_str(&value[literal_start..start]);
            if name != key && self.extra.contains_key(name) {
                match self.resolve(name) {
                    Some(replacement) => result.push_str(&replacement),
                    None => {
                        valid = false;
                        result.push_str(&value[start..end]);
                    }
                }
            } else if let Some(replacement) = self.inherited.get(name) {
                // Self-references extend the inherited value, e.g. PATH = "$PATH:/tools".
                result.push_str(replacement);
            } else {
                if name == key {
                    valid = false;
                } else {
                    missing.insert(name);
                }
                result.push_str(&value[start..end]);
            }
            offset = end;
            literal_start = end;
        }
        result.push_str(&value[literal_start..]);
        // A tilde inserted from an environment value must remain literal.
        if value == "~" || value.starts_with("~/") || value.starts_with("~\\") {
            match self.inherited.get("HOME").or_else(|| self.inherited.get("USERPROFILE")) {
                Some(home) if !home.is_empty() => result = format!("{home}{}", &result[1..]),
                _ => self.warnings.push(format!("lsp.cjls.binary.env.{key}: HOME and USERPROFILE do not provide a home directory; keeping ~")),
            }
        }
        if !missing.is_empty() {
            self.warnings.push(format!("lsp.cjls.binary.env.{key}: environment variables are not set: {}; keeping references", missing.into_iter().collect::<Vec<_>>().join(", ")));
        }

        self.active.remove(key);
        let result = if valid {
            Some(result)
        } else {
            self.warnings.push(format!("lsp.cjls.binary.env.{key}: cyclic entry reference or dependency; keeping configured value"));
            None
        };
        self.resolved.insert(key.to_string(), result.clone());
        result
    }
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
    fn inherited_values_stay_literal_and_overrides_take_precedence() {
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
                ("COPY", "/override"),
                ("TILDE", "~/sdk")
            ])
        );
        result.sort();
        other.sort();
        assert_eq!(result, other);
    }

    #[test]
    fn entry_chains_resolve_independently_of_order_and_empty_overrides_win() {
        let extra = pairs(&[
            ("STDX", "${SDK}/static/stdx"),
            ("SDK", "$ROOT/sdk"),
            ("ROOT", "$HOME"),
            ("EMPTY", ""),
            ("COPY", "$EMPTY"),
            ("PATH", "${PATH}:/tools"),
        ]);
        let inherited = pairs(&[
            ("HOME", "/home/user"),
            ("SDK", "/old"),
            ("EMPTY", "/old"),
            ("PATH", "/bin"),
        ]);
        let expected = pairs(&[
            ("STDX", "/home/user/sdk/static/stdx"),
            ("SDK", "/home/user/sdk"),
            ("ROOT", "/home/user"),
            ("EMPTY", ""),
            ("COPY", ""),
            ("PATH", "/bin:/tools"),
        ]);
        let result = expand(extra.clone(), &inherited, |message| panic!("{message}"));
        assert_eq!(result, expected);
        let mut reversed = extra;
        reversed.reverse();
        let mut result = expand(reversed, &inherited, |message| panic!("{message}"));
        result.reverse();
        assert_eq!(result, expected);
    }

    #[test]
    fn cycles_and_their_dependents_stay_unchanged_in_either_entry_order() {
        let extra = pairs(&[
            ("A", "secret-$B"),
            ("B", "${A}/suffix"),
            ("CHILD", "$A/child"),
            ("SELF", "$SELF"),
            ("OK", "$HOME/ok"),
        ]);
        let mut expected = extra.clone();
        expected.last_mut().unwrap().1 = "/home/ok".to_string();
        let inherited = pairs(&[("HOME", "/home"), ("A", "/old")]);
        let mut warnings = Vec::new();
        assert_eq!(
            expand(extra.clone(), &inherited, |message| warnings.push(message)),
            expected
        );
        assert_eq!(warnings.len(), 4);
        assert!(
            warnings
                .iter()
                .all(|message| message.contains("cyclic") && !message.contains("secret-"))
        );
        let mut reversed = extra;
        reversed.reverse();
        let mut result = expand(reversed, &inherited, |_| {});
        result.reverse();
        assert_eq!(result, expected);
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
