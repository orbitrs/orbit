#[cfg(test)]
mod tests {
    use crate::style::{CssProperty, CssSelector, Specificity, StyleRule, Stylesheet};

    #[test]
    fn test_specificity_calculation() {
        let rule = StyleRule::new(
            vec![CssSelector {
                selector: "#main .header h1".to_string(),
                properties: vec![],
            }],
            true,
            0,
        );
        assert_eq!(rule.specificity, Specificity(1, 1, 1));

        let rule = StyleRule::new(
            vec![CssSelector {
                selector: ".button.primary:hover".to_string(),
                properties: vec![],
            }],
            true,
            0,
        );
        assert_eq!(rule.specificity, Specificity(0, 3, 0));
    }

    #[test]
    fn test_css_parsing() {
        let css = r#"
            .button {
                background-color: blue;
                color: white;
            }

            .button:hover {
                background-color: darkblue;
            }
        "#;

        let result = Stylesheet::parse(css, true);
        assert!(result.is_ok());

        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules.len(), 2);

        let first_rule = &stylesheet.rules[0];
        assert_eq!(first_rule.selectors[0].selector, ".button");
        assert_eq!(first_rule.selectors[0].properties.len(), 2);
        assert_eq!(first_rule.scoped, true);
    }

    #[test]
    fn test_scoping() {
        let mut rule = StyleRule::new(
            vec![CssSelector {
                selector: ".button".to_string(),
                properties: vec![],
            }],
            true,
            0,
        );

        rule.apply_scoping("component-123");
        assert_eq!(rule.selectors[0].selector, ".component-123 .button");
        assert_eq!(rule.specificity, Specificity(0, 2, 0));
    }

    #[test]
    fn test_multiple_selectors() {
        let css = r#"
            h1, h2, h3 {
                font-family: sans-serif;
            }

            .alert, .notice {
                padding: 1rem;
                border: 1px solid;
            }
        "#;

        let result = Stylesheet::parse(css, false);
        assert!(result.is_ok());

        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules.len(), 2);

        let first_rule = &stylesheet.rules[0];
        assert_eq!(first_rule.selectors.len(), 3);

        let second_rule = &stylesheet.rules[1];
        assert_eq!(second_rule.selectors.len(), 2);
        assert_eq!(second_rule.selectors[0].properties.len(), 2);
    }
}
