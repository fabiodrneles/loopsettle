//! Minimal JSON writer (DECISIONS.md D1). Produces compact, deterministic output:
//! keys keep insertion order, floats use a fixed formatting rule.

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i128),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn obj() -> Self {
        Json::Obj(Vec::new())
    }
    /// Builder-style insertion for objects: `Json::obj().set("a", 1)`.
    pub fn set(mut self, k: &str, v: impl Into<Json>) -> Self {
        if let Json::Obj(ref mut kv) = self {
            kv.push((k.to_string(), v.into()));
        }
        self
    }
    pub fn push(&mut self, k: &str, v: impl Into<Json>) {
        if let Json::Obj(ref mut kv) = self {
            kv.push((k.to_string(), v.into()));
        }
    }

    pub fn to_string_pretty(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, 0, true);
        s
    }
    pub fn to_string_compact(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, 0, false);
        s
    }

    fn write(&self, s: &mut String, indent: usize, pretty: bool) {
        let nl = |s: &mut String, i: usize| {
            if pretty {
                s.push('\n');
                for _ in 0..i {
                    s.push_str("  ");
                }
            }
        };
        match self {
            Json::Null => s.push_str("null"),
            Json::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
            Json::Int(i) => s.push_str(&i.to_string()),
            Json::Num(x) => s.push_str(&fmt_f64(*x)),
            Json::Str(t) => write_str(s, t),
            Json::Arr(v) => {
                s.push('[');
                let flat = v.iter().all(|x| !matches!(x, Json::Arr(_) | Json::Obj(_)));
                for (i, x) in v.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    if !flat {
                        nl(s, indent + 1);
                    }
                    x.write(s, indent + 1, pretty);
                }
                if !flat && !v.is_empty() {
                    nl(s, indent);
                }
                s.push(']');
            }
            Json::Obj(kv) => {
                s.push('{');
                for (i, (k, v)) in kv.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    nl(s, indent + 1);
                    write_str(s, k);
                    s.push(':');
                    if pretty {
                        s.push(' ');
                    }
                    v.write(s, indent + 1, pretty);
                }
                if !kv.is_empty() {
                    nl(s, indent);
                }
                s.push('}');
            }
        }
    }
}

/// Floats: non-finite -> null; otherwise up to 6 significant decimals, trailing zeros cut.
pub fn fmt_f64(x: f64) -> String {
    if !x.is_finite() {
        return "null".into();
    }
    if x == x.trunc() && x.abs() < 1e15 {
        return format!("{}", x as i64);
    }
    let s = format!("{:.6}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

fn write_str(s: &mut String, t: &str) {
    s.push('"');
    for c in t.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            c if (c as u32) < 0x20 => s.push_str(&format!("\\u{:04x}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
}

impl From<bool> for Json {
    fn from(b: bool) -> Self {
        Json::Bool(b)
    }
}
impl From<f64> for Json {
    fn from(x: f64) -> Self {
        Json::Num(x)
    }
}
impl From<&str> for Json {
    fn from(x: &str) -> Self {
        Json::Str(x.into())
    }
}
impl From<String> for Json {
    fn from(x: String) -> Self {
        Json::Str(x)
    }
}
impl From<Vec<Json>> for Json {
    fn from(x: Vec<Json>) -> Self {
        Json::Arr(x)
    }
}
// A macro avoids writing the same `From` impl for every integer type.
macro_rules! from_int {
    ($($t:ty),*) => { $(impl From<$t> for Json { fn from(x: $t) -> Self { Json::Int(x as i128) } })* };
}
from_int!(i32, i64, i128, u32, u64, usize);

impl<T: Into<Json> + Clone> From<&[T]> for Json {
    fn from(x: &[T]) -> Self {
        Json::Arr(x.iter().cloned().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn writes_nested() {
        let j = Json::obj().set("a", 1i64).set("b", 0.5).set("c", "x\"y").set("d", &[1i64, 2][..]);
        assert_eq!(j.to_string_compact(), r#"{"a":1,"b":0.5,"c":"x\"y","d":[1,2]}"#);
    }
}
