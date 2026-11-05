//! A tiny parser for the subset of TOML used by `experiments/*.toml` and
//! `data/calibration/*.toml` (DECISIONS.md D1):
//! `[section]` headers, `key = value` with integers, floats, booleans, "strings" and
//! single-line arrays of those; `#` comments. Keys are flattened to `section.key`.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Arr(Vec<Value>),
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Doc {
    pub map: BTreeMap<String, Value>,
}

impl Doc {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }
    pub fn f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(Value::as_f64)
    }
    pub fn i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(Value::as_i64)
    }
    pub fn str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }
    pub fn f64_array(&self, key: &str) -> Option<Vec<f64>> {
        match self.get(key)? {
            Value::Arr(v) => v.iter().map(Value::as_f64).collect(),
            _ => None,
        }
    }
    pub fn i64_array(&self, key: &str) -> Option<Vec<i64>> {
        match self.get(key)? {
            Value::Arr(v) => v.iter().map(Value::as_i64).collect(),
            _ => None,
        }
    }
    pub fn str_array(&self, key: &str) -> Option<Vec<String>> {
        match self.get(key)? {
            Value::Arr(v) => v.iter().map(|x| x.as_str().map(String::from)).collect(),
            _ => None,
        }
    }
}

pub fn parse(src: &str) -> Result<Doc, String> {
    let mut doc = Doc::default();
    let mut section = String::new();
    for (no, raw) in src.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let err = |m: &str| format!("line {}: {m}: {raw}", no + 1);
        if let Some(rest) = line.strip_prefix('[') {
            section = rest.strip_suffix(']').ok_or_else(|| err("bad section header"))?.trim().to_string();
            continue;
        }
        let (k, v) = line.split_once('=').ok_or_else(|| err("expected key = value"))?;
        let key = if section.is_empty() { k.trim().to_string() } else { format!("{section}.{}", k.trim()) };
        doc.map.insert(key, parse_value(v.trim()).map_err(|e| err(&e))?);
    }
    Ok(doc)
}

fn strip_comment(s: &str) -> &str {
    let mut in_str = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '#' if !in_str => return &s[..i],
            _ => {}
        }
    }
    s
}

fn parse_value(v: &str) -> Result<Value, String> {
    if let Some(inner) = v.strip_prefix('[') {
        let inner = inner.strip_suffix(']').ok_or("unterminated array")?;
        let items = split_top(inner);
        return items.iter().filter(|s| !s.trim().is_empty()).map(|s| parse_value(s.trim())).collect::<Result<_, _>>().map(Value::Arr);
    }
    if let Some(s) = v.strip_prefix('"') {
        return Ok(Value::Str(s.strip_suffix('"').ok_or("unterminated string")?.to_string()));
    }
    match v {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    let clean = v.replace('_', "");
    if let Ok(i) = clean.parse::<i64>() {
        return Ok(Value::Int(i));
    }
    clean.parse::<f64>().map(Value::Float).map_err(|_| format!("cannot parse value '{v}'"))
}

fn split_top(s: &str) -> Vec<&str> {
    let (mut out, mut depth, mut in_str, mut start) = (Vec::new(), 0, false, 0);
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '[' if !in_str => depth += 1,
            ']' if !in_str => depth -= 1,
            ',' if !in_str && depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_subset() {
        let d = parse("a = 1 # c\n[s]\nx = 2.5\nname = \"hi # there\"\narr = [1, 2_000, 3]\nb = true\n").unwrap();
        assert_eq!(d.i64("a"), Some(1));
        assert_eq!(d.f64("s.x"), Some(2.5));
        assert_eq!(d.str("s.name"), Some("hi # there"));
        assert_eq!(d.i64_array("s.arr"), Some(vec![1, 2000, 3]));
        assert_eq!(d.get("s.b"), Some(&Value::Bool(true)));
    }
}
