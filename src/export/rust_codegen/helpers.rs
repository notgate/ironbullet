use std::collections::HashSet;

use crate::pipeline::block::*;

pub(super) struct VarTracker {
    defined: HashSet<String>,
}

impl VarTracker {
    pub fn new() -> Self {
        Self { defined: HashSet::new() }
    }

    pub fn define(&mut self, name: &str) {
        self.defined.insert(var_name(name));
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.defined.contains(&var_name(name))
    }

    /// Returns "let " if variable is new, "" if reassigning
    pub fn let_or_assign(&mut self, name: &str) -> &'static str {
        let vn = var_name(name);
        if self.defined.contains(&vn) {
            ""
        } else {
            self.defined.insert(vn);
            "let "
        }
    }
}

pub(super) fn generate_condition_code(cond: &KeyCondition) -> String {
    let source = if cond.source == "data.RESPONSECODE" {
        "status_code".to_string()
    } else {
        var_name(&cond.source)
    };

    match cond.comparison {
        Comparison::Contains => format!("source.contains(\"{}\")", escape_str(&cond.value)),
        Comparison::NotContains => format!("!source.contains(\"{}\")", escape_str(&cond.value)),
        Comparison::EqualTo => {
            if cond.source == "data.RESPONSECODE" {
                format!("{} == {}", source, cond.value)
            } else {
                format!("{} == \"{}\"", source, escape_str(&cond.value))
            }
        }
        Comparison::NotEqualTo => format!("{} != \"{}\"", source, escape_str(&cond.value)),
        Comparison::GreaterThan => format!("{} > {}", source, cond.value),
        Comparison::LessThan => format!("{} < {}", source, cond.value),
        Comparison::MatchesRegex => format!("Regex::new(r\"{}\").unwrap().is_match(&{})", escape_str(&cond.value), source),
        Comparison::Exists => format!("!{}.is_empty()", source),
        Comparison::NotExists => format!("{}.is_empty()", source),
    }
}

pub(super) fn var_name(s: &str) -> String {
    s.replace('.', "_")
        .replace('@', "")
        .to_lowercase()
}

pub(super) fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
