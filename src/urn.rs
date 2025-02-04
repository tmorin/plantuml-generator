use std::fmt;
use std::str::FromStr;

use heck::ToTitleCase;
use schemars::JsonSchema;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, JsonSchema)]
pub struct Urn {
    /// The URN value.
    pub value: String,
    /// The latest component of the value.
    pub name: String,
    /// The latest component of the value.
    pub label: String,
    /// The relative path to the library's directory.
    pub path_to_base: String,
}

impl PartialEq for Urn {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Urn {
    pub fn get_parent(&self) -> Urn {
        let parts: Vec<&str> = self.value.split('/').collect();
        if parts.len() == 1 {
            return self.clone();
        }
        let parent_parts: Vec<&str> = parts.iter().take(parts.len() - 1).copied().collect();
        let parent_urn_value: String = parent_parts.join("/");
        Urn::from(parent_urn_value.as_str())
    }
    pub fn is_included_in(&self, urns: &[Urn]) -> bool {
        urns.is_empty()
            || urns.iter().any(|other| {
                // OK if descendant
                if other.value.len() <= self.value.len() && self.value.starts_with(&other.value) {
                    return true;
                }
                // OK if ancestor
                other.value.starts_with(&self.value)
            })
    }
}

impl fmt::Display for Urn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Serialize for Urn {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for Urn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UrnVisitor;

        impl Visitor<'_> for UrnVisitor {
            type Value = Urn;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string <p>[/<p>]+")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Urn::from(v))
            }
        }
        deserializer.deserialize_string(UrnVisitor)
    }
}

impl FromStr for Urn {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Urn::from(s))
    }
}

impl From<&str> for Urn {
    fn from(value: &str) -> Self {
        let parts: Vec<&str> = value.split('/').map(|_| "..").collect();
        let path_to_base = match parts.is_empty() {
            true => ".".to_string(),
            false => parts.join("/"),
        };

        let name = value.split('/').last().unwrap_or(value);
        Urn {
            value: String::from(value),
            name: String::from(name),
            label: String::from(name).to_title_case(),
            path_to_base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_base() {
        assert_eq!(
            Urn::from("PackageA/ModuleB/FamilyC/ItemD").path_to_base,
            "../../../.."
        );
        assert_eq!(
            Urn::from("PackageA/ModuleB/FamilyC").path_to_base,
            "../../.."
        );
        assert_eq!(Urn::from("PackageA/ModuleB").path_to_base, "../..");
        assert_eq!(Urn::from("PackageA").path_to_base, "..");
    }

    #[test]
    fn test_urn() {
        let urn = Urn::from("PackageA/ModuleB/FamilyC/ItemD");
        assert_eq!(urn.value, "PackageA/ModuleB/FamilyC/ItemD");
        assert_eq!(urn.name, "ItemD");
        assert_eq!(urn.label, "Item D");
        assert_eq!(urn.get_parent().value, "PackageA/ModuleB/FamilyC");
        assert_eq!(urn.get_parent().get_parent().value, "PackageA/ModuleB");
        assert_eq!(urn.get_parent().get_parent().get_parent().value, "PackageA");
        assert_eq!(
            urn.get_parent()
                .get_parent()
                .get_parent()
                .get_parent()
                .value,
            "PackageA"
        );
    }

    #[test]
    fn test_urn_is_included_in_c4model() {
        assert!(Urn::from("c4model").is_included_in(&[Urn::from("c4model/Element")]));
        assert!(Urn::from("c4model/Element/Person/External")
            .is_included_in(&[Urn::from("c4model/Element/Person")]));
        assert!(Urn::from("c4model/Element/Person").is_included_in(&[Urn::from("c4model/Element")]));
        assert!(!Urn::from("c4model/Category").is_included_in(&[Urn::from("c4model/Element")]));
    }

    #[test]
    fn test_urn_is_included_in() {
        assert!(
            Urn::from("PackageA/ModuleB/FamilyC/ItemD").is_included_in(&[Urn::from("PackageA")])
        );
        assert!(Urn::from("PackageA/ModuleB/FamilyC/ItemD")
            .is_included_in(&[Urn::from("PackageA/ModuleB/FamilyC")]));
        assert!(Urn::from("PackageA/ModuleB/FamilyC/ItemD")
            .is_included_in(&[Urn::from("PackageA/ModuleB/FamilyC/ItemD")]));
        assert!(
            !Urn::from("PackageA/ModuleB/FamilyC/ItemD").is_included_in(&[Urn::from("PackageBis")])
        );
        assert!(!Urn::from("PackageA/ModuleB/FamilyC/ItemD")
            .is_included_in(&[Urn::from("PackageA/ModuleB/FamilyBis")]));
        assert!(!Urn::from("PackageA/ModuleB/FamilyC/ItemD")
            .is_included_in(&[Urn::from("PackageA/ModuleB/FamilyC/ItemBis")]));
        assert!(Urn::from("PackageA/ModuleB/FamilyC/ItemD")
            .is_included_in(&[Urn::from("PackageA/ModuleB/FamilyC/ItemD/Bis")]));
        assert!(Urn::from("PackageA").is_included_in(&[Urn::from("PackageA")]));
        assert!(!Urn::from("PackageB").is_included_in(&[Urn::from("PackageA")]));
        assert!(Urn::from("PackageA").is_included_in(&[Urn::from("PackageA")]));
    }
}
