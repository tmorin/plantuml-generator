use std::str::FromStr;

use tera::Tera;

use crate::error::Error;
use crate::plantuml::PlantUML;
use crate::result::Result;

#[derive(Eq, PartialEq)]
pub enum CleanupScope {
    All,
    Example,
    Item,
    ItemIcon,
    ItemSource,
    Snippet,
    SnippetSource,
    SnippetImage,
    Sprite,
    SpriteIcon,
    SpriteValue,
}

impl FromStr for CleanupScope {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "All" => Ok(CleanupScope::All),
            "Example" => Ok(CleanupScope::Example),
            "Item" => Ok(CleanupScope::Item),
            "ItemIcon" => Ok(CleanupScope::ItemIcon),
            "ItemSource" => Ok(CleanupScope::ItemSource),
            "Snippet" => Ok(CleanupScope::Snippet),
            "SnippetSource" => Ok(CleanupScope::SnippetSource),
            "SnippetImage" => Ok(CleanupScope::SnippetImage),
            "Sprite" => Ok(CleanupScope::Sprite),
            "SpriteIcon" => Ok(CleanupScope::SpriteIcon),
            "SpriteValue" => Ok(CleanupScope::SpriteValue),
            _ => Err(Error::Simple(format!("unable to find a match for {}", s))),
        }
    }
}

impl CleanupScope {
    pub fn is_included_in(&self, scopes: &[CleanupScope]) -> bool {
        if scopes.contains(self) {
            // true when the expected scope is part of the current
            return true;
        }
        match self {
            CleanupScope::ItemIcon => {
                scopes.contains(&CleanupScope::All) || scopes.contains(&CleanupScope::Item)
            }
            CleanupScope::ItemSource => {
                scopes.contains(&CleanupScope::All) || scopes.contains(&CleanupScope::Item)
            }
            CleanupScope::SnippetSource => {
                scopes.contains(&CleanupScope::All)
                    || scopes.contains(&CleanupScope::Item)
                    || scopes.contains(&CleanupScope::Snippet)
            }
            CleanupScope::SnippetImage => {
                scopes.contains(&CleanupScope::All)
                    || scopes.contains(&CleanupScope::Item)
                    || scopes.contains(&CleanupScope::Snippet)
            }
            CleanupScope::SpriteIcon => {
                scopes.contains(&CleanupScope::All)
                    || scopes.contains(&CleanupScope::Item)
                    || scopes.contains(&CleanupScope::Sprite)
            }
            CleanupScope::SpriteValue => {
                scopes.contains(&CleanupScope::All)
                    || scopes.contains(&CleanupScope::Item)
                    || scopes.contains(&CleanupScope::Sprite)
            }
            _ => scopes.contains(&CleanupScope::All),
        }
    }
}

pub trait Task {
    fn cleanup(&self, _scopes: &[CleanupScope]) -> Result<()> {
        Ok(())
    }
    fn create_resources(&self) -> Result<()> {
        Ok(())
    }
    fn render_atomic_templates(&self, _tera: &Tera) -> Result<()> {
        Ok(())
    }
    fn render_composed_templates(&self, _tera: &Tera) -> Result<()> {
        Ok(())
    }
    fn render_sources(&self, _plantuml: &PlantUML) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope() {
        assert!(CleanupScope::All.is_included_in(&[CleanupScope::All]));
        assert!(CleanupScope::Example.is_included_in(&[CleanupScope::Example]));
        assert!(CleanupScope::Example.is_included_in(&[CleanupScope::All]));
        assert!(CleanupScope::Item.is_included_in(&[CleanupScope::All, CleanupScope::Item]));
        assert!(CleanupScope::ItemSource.is_included_in(&[CleanupScope::Item]));
        assert!(CleanupScope::ItemIcon.is_included_in(&[CleanupScope::Item]));
        assert!(!CleanupScope::Item.is_included_in(&[CleanupScope::ItemSource]));
    }
}
