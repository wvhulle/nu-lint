pub mod best_practices;
pub mod documentation;
pub mod performance;
pub mod style;
pub mod type_safety;

use crate::rule::Rule;
use std::collections::HashMap;

pub struct RuleRegistry {
    rules: HashMap<String, Box<dyn Rule>>,
}

impl RuleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        let id = rule.id().to_string();
        self.rules.insert(id, rule);
    }

    #[must_use]
    pub fn get_rule(&self, id: &str) -> Option<&dyn Rule> {
        self.rules.get(id).map(std::convert::AsRef::as_ref)
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.values().map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn with_default_rules() -> Self {
        let mut registry = Self::new();

        registry.register(Box::<style::SnakeCaseVariables>::default());
        registry.register(Box::<style::KebabCaseCommands>::default());
        registry.register(Box::<style::ScreamingSnakeConstants>::default());
        registry.register(Box::new(style::PipeSpacing));
        registry.register(Box::new(style::BraceSpacing));
        registry.register(Box::new(style::PreferCompoundAssignment::new()));
        registry.register(Box::new(style::UnnecessaryVariableBeforeReturn::new()));
        registry.register(Box::new(style::PreferIsNotEmpty));
        registry.register(Box::new(style::DiscouragedBareIgnore::new()));
        registry.register(Box::new(style::DiscourageUnderscoreCommands::new()));
        registry.register(Box::new(style::CompletionFunctionNaming::new()));
        registry.register(Box::new(style::UnnecessaryMut::new()));

        registry.register(Box::new(best_practices::PreferErrorMake::new()));
        registry.register(Box::new(best_practices::AvoidMutableAccumulation));
        registry.register(Box::new(best_practices::PreferRangeIteration::new()));
        registry.register(Box::new(best_practices::PreferParseCommand::new()));
        registry.register(Box::new(best_practices::ConsistentErrorHandling::new()));
        registry.register(Box::new(best_practices::PreferMatchOverIfChain::new()));
        registry.register(Box::new(best_practices::PreferEachOverFor::new()));
        registry.register(Box::new(best_practices::DescriptiveErrorMessages::new()));
        registry.register(Box::new(
            best_practices::PreferBuiltinForCommonCommands::new(),
        ));
        registry.register(Box::new(best_practices::PreferBuiltinTextTransforms::new()));
        registry.register(Box::new(best_practices::PreferBuiltinSystemCommands::new()));

        registry.register(Box::new(performance::PreferWhereOverEachIf));
        registry.register(Box::new(performance::PreferLinesOverSplit::new()));
        registry.register(Box::new(performance::PreferParseOverEachSplit));

        registry.register(Box::new(documentation::MissingCommandDocs::new()));
        registry.register(Box::new(documentation::ExportedFunctionDocs));

        registry.register(Box::new(type_safety::MissingTypeAnnotation::new()));

        registry
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
