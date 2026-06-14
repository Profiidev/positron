use std::{cmp::Ordering, convert::Infallible, fmt::Display, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, JsonSchema)]
pub struct Scope(Vec<String>);

impl Scope {
  fn from_str_internal(input: &str) -> Self {
    let value = input.split(" ").map(str::to_string);
    Self(value.collect())
  }

  pub fn intersect(&self, other: &Self) -> Scope {
    Self(
      self
        .0
        .iter()
        .filter(|&s| other.0.contains(s))
        .cloned()
        .collect(),
    )
  }

  #[inline]
  fn to_string_internal(&self) -> String {
    self.0.join(" ")
  }

  #[inline]
  fn overlapping_count(&self, other: &Self) -> usize {
    self.0.iter().filter(|&s| other.0.contains(s)).count()
  }

  #[inline]
  fn len(&self) -> usize {
    self.0.len()
  }

  #[inline]
  fn greater_eq(&self, other: &Self) -> bool {
    self.overlapping_count(other) == other.len()
  }

  #[inline]
  fn greater(&self, other: &Self) -> bool {
    self.greater_eq(other) && self.len() > other.len()
  }

  #[inline]
  pub fn contains(&self, scope: &str) -> bool {
    self.0.iter().any(|s| s == scope)
  }

  #[inline]
  pub fn inner(&self) -> &[String] {
    &self.0
  }
}

impl From<Vec<String>> for Scope {
  fn from(value: Vec<String>) -> Self {
    Self(value)
  }
}

impl FromStr for Scope {
  type Err = Infallible;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from_str_internal(s))
  }
}

impl Display for Scope {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string_internal())
  }
}

impl Serialize for Scope {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for Scope {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
      String(String),
      Vec(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
      StringOrVec::String(s) => {
        let Ok(scope) = s.parse::<Scope>();
        Ok(scope)
      }
      StringOrVec::Vec(v) => Ok(Scope(v)),
    }
  }
}

impl PartialEq for Scope {
  fn eq(&self, other: &Self) -> bool {
    self.overlapping_count(other) == self.len() && self.len() == other.len()
  }
}

impl PartialOrd for Scope {
  fn ge(&self, other: &Self) -> bool {
    self.greater_eq(other)
  }

  fn gt(&self, other: &Self) -> bool {
    self.greater(other)
  }

  fn le(&self, other: &Self) -> bool {
    !self.greater(other)
  }

  fn lt(&self, other: &Self) -> bool {
    !self.greater_eq(other)
  }

  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(if self.eq(other) {
      Ordering::Equal
    } else if self.greater(other) {
      Ordering::Greater
    } else {
      Ordering::Less
    })
  }
}

#[cfg(test)]
mod test {
  use super::Scope;
  use std::cmp::Ordering;
  use std::str::FromStr;

  fn s(parts: &[&str]) -> Scope {
    Scope(parts.iter().map(|p| p.to_string()).collect())
  }

  #[test]
  fn from_str_splits_on_space() {
    let scope = Scope::from_str("openid profile email").unwrap();
    assert_eq!(scope.inner(), ["openid", "profile", "email"]);
  }

  #[test]
  fn from_str_empty_yields_single_empty_element() {
    // "".split(' ') yields one empty string
    let scope = Scope::from_str("").unwrap();
    assert_eq!(scope.inner(), [""]);
  }

  #[test]
  fn display_roundtrips_with_from_str() {
    let scope = Scope::from_str("a b c").unwrap();
    assert_eq!(scope.to_string(), "a b c");
  }

  #[test]
  fn from_vec() {
    let scope = Scope::from(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(scope.inner(), ["a", "b"]);
  }

  #[test]
  fn contains_checks_membership() {
    let scope = s(&["openid", "email"]);
    assert!(scope.contains("openid"));
    assert!(!scope.contains("profile"));
  }

  #[test]
  fn intersect_keeps_common_elements_in_self_order() {
    let a = s(&["openid", "profile", "email"]);
    let b = s(&["email", "openid"]);
    assert_eq!(a.intersect(&b).inner(), ["openid", "email"]);
    // disjoint -> empty
    assert!(s(&["x"]).intersect(&s(&["y"])).inner().is_empty());
  }

  #[test]
  fn equality_is_set_like() {
    assert_eq!(s(&["a", "b"]), s(&["a", "b"]));
    // different length
    assert_ne!(s(&["a", "b"]), s(&["a"]));
    // same length, different members
    assert_ne!(s(&["a", "b"]), s(&["a", "c"]));
  }

  #[test]
  fn ordering_superset_is_greater() {
    let big = s(&["a", "b", "c"]);
    let small = s(&["a", "b"]);
    assert!(big > small);
    assert!(big >= small);
    assert!(small < big);
    assert!(small <= big);
  }

  #[test]
  fn ordering_equal_sets() {
    let a = s(&["a", "b"]);
    let b = s(&["b", "a"]);
    assert!(a >= b);
    assert!(a <= b);
    // equal sets are neither strictly greater nor strictly less (method form
    // keeps clippy's neg_cmp_op_on_partial_ord lint happy)
    assert!(!a.gt(&b));
    assert!(!a.lt(&b));
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
  }

  #[test]
  fn ordering_disjoint_sets_are_less() {
    let a = s(&["a"]);
    let b = s(&["b"]);
    // neither equal nor superset -> partial_cmp returns Less by definition
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
  }

  #[test]
  fn partial_cmp_greater_branch() {
    assert_eq!(
      s(&["a", "b"]).partial_cmp(&s(&["a"])),
      Some(Ordering::Greater)
    );
  }

  #[test]
  fn serialize_is_space_joined_string() {
    let scope = s(&["openid", "email"]);
    assert_eq!(serde_json::to_string(&scope).unwrap(), "\"openid email\"");
  }

  #[test]
  fn deserialize_from_string() {
    let scope: Scope = serde_json::from_str("\"openid email\"").unwrap();
    assert_eq!(scope.inner(), ["openid", "email"]);
  }

  #[test]
  fn deserialize_from_array() {
    let scope: Scope = serde_json::from_str("[\"openid\",\"email\"]").unwrap();
    assert_eq!(scope.inner(), ["openid", "email"]);
  }

  #[test]
  fn default_is_empty() {
    assert!(Scope::default().inner().is_empty());
  }
}
