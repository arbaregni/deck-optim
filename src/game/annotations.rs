use serde::Deserialize;
use serde::Serialize;

use crate::game::ManaPool;

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

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct AnnotationTarget {
    /// A list of card names to apply this annotation to
    pub targets: Vec<String>,

    // The annotations to apply
    #[serde(flatten)]
    pub annotation: Annotation,
}


#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct Annotation {
    /// The name of this annotation
    pub key: String,
    /// The value of this annotation, leave blank if it does not matter
    pub value: Option<AnnotationValue>,
}

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub enum AnnotationValue {
    String(String),
    Mana(ManaPool)
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
                        value: None
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
            "value": {
                "Mana": "{G}"
            }
        }] }
        "#;

        let expected = CardAnnotations {
            annotations: vec![
                AnnotationTarget {
                    targets: vec!["Forest".to_string()],
                    annotation: Annotation {
                        key: "produces".to_string(),
                        value: Some(AnnotationValue::Mana(ManaPool {
                            green: 1,
                            ..Default::default()
                        }))
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
            "value": {
                "String": "go-wide"
            }
        }]}
        "#;

        let expected = CardAnnotations {
            annotations: vec![
                AnnotationTarget {
                    targets: vec!["Craterhoof Behemoth".to_string()],
                    annotation: Annotation {
                        key: "payoff".to_string(),
                        value: Some(AnnotationValue::String(
                                "go-wide".to_string()
                        ))
                    }
                }
            ]
        };
        
        let actual: CardAnnotations = serde_json::from_str(source).expect("no errors");

        assert_eq!(actual, expected);
    }


}
