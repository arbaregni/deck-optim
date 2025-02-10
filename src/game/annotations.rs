use serde::Deserialize;
use serde::Serialize;

use crate::game::ManaPool;

/// A list of annotations to apply to particular targets
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct CardAnnotations {
    annotations: Vec<AnnotationTarget>
}
impl CardAnnotations {
    pub fn into_iter(self) -> impl Iterator<Item = AnnotationTarget> {
        self.annotations.into_iter()
    }
    pub fn len(&self) -> usize {
        self.annotations.len()
    }
}

/// An annotation being applied to a list of targets, which are Card Names
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct AnnotationTarget {
    /// A list of card names to apply this annotation to
    pub targets: Vec<String>,

    // The annotations to apply
    #[serde(flatten)]
    pub annotation: Annotation,
}

impl AnnotationTarget {
    pub fn targets(&self) -> &[String] {
        self.targets.as_slice()
    }
    pub fn annotation(&self) -> &Annotation {
        &self.annotation
    }
}


/// An annotation is a key-value pair.
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct Annotation {
    /// The name of this annotation. Annotation keys starting with "core:" are reserved 
    /// for the engine to use.
    pub key: String,

    /// The value of this annotation, leave blank if it does not matter
    #[serde(default = "Vec::new")]
    pub values: Vec<AnnotationValue>,
}
impl Annotation {
    pub fn key(&self) -> &str {
        self.key.as_str()
    }
    pub fn values(&self) -> &[AnnotationValue] {
        self.values.as_slice()
    }
    pub fn extend(&mut self, values: Vec<AnnotationValue>) {
        self.values.extend(values);
        // remove duplicates
        self.values.sort();
        self.values.dedup();
    }
}

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq,PartialOrd,Ord)]
pub enum AnnotationValue {
    String(String),
    Mana(ManaPool)
}


#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct AnnotationSet {
    annotations: Vec<Annotation>
}
impl AnnotationSet {
    /// An empty set of annotations
    pub const fn empty() -> Self {
        Self { annotations: Vec::new() }
    }
    /// Insert another annotation into this annotation set.
    ///
    /// # Ex
    ///
    /// ```
    /// use deck_optim::game::annotations::{
    ///     Annotation, AnnotationValue, AnnotationSet
    /// };
    ///
    /// let mut original = AnnotationSet::from([
    ///     Annotation {
    ///         key: "foo".to_string(),
    ///         values: vec![
    ///             AnnotationValue::String("bar".to_string())
    ///         ]
    ///     }
    /// ]);
    ///
    /// original.insert(Annotation {
    ///     key: "baz".to_string(),
    ///     values: vec![]
    /// });
    ///
    /// assert_eq!(original, AnnotationSet::from([
    ///     Annotation {
    ///         key: "foo".to_string(),
    ///         values: vec![
    ///             AnnotationValue::String("bar".to_string())
    ///         ]
    ///     },
    ///     Annotation {
    ///         key: "baz".to_string(),
    ///         values: vec![]
    ///     }
    /// ]));
    ///
    /// original.insert(Annotation {
    ///     key: "foo".to_string(),
    ///     values: vec![AnnotationValue::String("bar".to_string()), AnnotationValue::String("quux".to_string())]
    /// });
    ///
    /// assert_eq!(original, AnnotationSet::from([
    ///     Annotation {
    ///         key: "foo".to_string(),
    ///         values: vec![
    ///             AnnotationValue::String("bar".to_string()),
    ///             AnnotationValue::String("quux".to_string())
    ///         ]
    ///     },
    ///     Annotation {
    ///         key: "baz".to_string(),
    ///         values: vec![]
    ///     }
    /// ]));
    /// 
    pub fn insert(&mut self, annotation: Annotation) {
        let entry = self.get_mut(annotation.key.as_str());
        match entry {
            None => self.annotations.push(annotation),
            Some(entry) => {
                entry.extend(annotation.values);
            }
        }
    }
    /// Lookup a key in the annotation set
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<&Annotation> {
        let idx = self.annotations.iter().position(|an| an.key == key.as_ref())?;
        Some(&self.annotations[idx])
    }
    /// Mutably lookup a key in the annotation set
    pub fn get_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut Annotation> {
        let idx = self.annotations.iter().position(|an| an.key == key.as_ref())?;
        Some(&mut self.annotations[idx])
    }
}
impl Default for AnnotationSet {
    /// An empty set of annotations
    fn default() -> Self {
        Self::empty()
    }
}
impl <Iter: IntoIterator<Item = Annotation>> From<Iter> for AnnotationSet {
    /// Create an annotation set from an iterator over [`Annotation`]s.
    fn from(value: Iter) -> Self {
        let mut this = Self::empty();
        for an in value.into_iter() {
            this.insert(an);
        }
        this
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_deser_with_no_value() {

        let source = r#"
        { "annotations": [{
            "targets": ["Rhystic Study", "Fathom Mage"],
            "key": "draw-engine"
        }] }
        "#;

        let expected = CardAnnotations {
            annotations: vec![
                AnnotationTarget {
                    targets: vec!["Rhystic Study".to_string(), "Fathom Mage".to_string()],
                    annotation: Annotation {
                        key: "draw-engine".to_string(),
                        values: vec![],
                    }
                }
            ]
        };
        
        println!("source: {source}");
        let actual: CardAnnotations = serde_json::from_str(source).expect("no errors");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deser_with_mana_value() {

        let source = r#"
        { "annotations": [{
            "targets": ["Forest"],
            "key": "produces",
            "values": [{
                "Mana": "{G}"
            }]
        }] }
        "#;

        let expected = CardAnnotations {
            annotations: vec![
                AnnotationTarget {
                    targets: vec!["Forest".to_string()],
                    annotation: Annotation {
                        key: "produces".to_string(),
                        values: vec![AnnotationValue::Mana(ManaPool {
                            green: 1,
                            ..Default::default()
                        })]
                    }
                }
            ]
        };
        
        let actual: CardAnnotations = serde_json::from_str(source).expect("no errors");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deser_with_str_value() {

        let source = r#"
        { "annotations": [{
            "targets": ["Craterhoof Behemoth"],
            "key": "payoff",
            "values": [{
                "String": "go-wide"
            }]
        }]}
        "#;

        let expected = CardAnnotations {
            annotations: vec![
                AnnotationTarget {
                    targets: vec!["Craterhoof Behemoth".to_string()],
                    annotation: Annotation {
                        key: "payoff".to_string(),
                        values: vec![AnnotationValue::String(
                                "go-wide".to_string()
                        )]
                    }
                }
            ]
        };
        
        let actual: CardAnnotations = serde_json::from_str(source).expect("no errors");

        assert_eq!(actual, expected);
    }


}
