use std::{cmp::Ordering, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone)]
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

  pub fn iter(&self) -> impl Iterator<Item = &str> {
    self.0.iter().map(AsRef::as_ref)
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
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
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
}

impl FromStr for Scope {
  type Err = ();
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
    let s = String::deserialize(deserializer)?;
    Ok(s.parse().unwrap())
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
