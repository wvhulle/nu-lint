pub mod avoid_mutable_accumulation;
pub mod brace_spacing;
pub mod completion_function_naming;
pub mod consistent_error_handling;
pub mod descriptive_error_messages;
pub mod discourage_bare_ignore;
pub mod discourage_underscore_commands;
pub mod exported_function_docs;
pub mod kebab_case_commands;
pub mod max_positional_params;
pub mod missing_command_docs;
pub mod missing_type_annotation;
pub mod multiline_formatting;
pub mod no_trailing_spaces;
pub mod omit_list_commas;
pub mod pipe_spacing;
pub mod prefer_builtin_commands;
pub mod prefer_builtin_system_commands;
pub mod prefer_builtin_text_transforms;
pub mod prefer_compound_assignment;
pub mod prefer_each_over_for;
pub mod prefer_error_make;
pub mod prefer_is_not_empty;
pub mod prefer_lines_over_split;
pub mod prefer_match_over_if_chain;
pub mod prefer_parse_command;
pub mod prefer_parse_over_each_split;
pub mod prefer_range_iteration;
pub mod prefer_where_over_each_if;
pub mod screaming_snake_constants;
pub mod snake_case_variables;
pub mod unnecessary_mut;
pub mod unnecessary_variable_before_return;

use std::collections::HashMap;

use crate::rule::{Rule, RuleMetadata};

pub struct RuleRegistry {
    rules: HashMap<String, Rule>,
}

impl RuleRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn register(&mut self, rule: Rule) {
        let id = rule.id().to_string();
        self.rules.insert(id, rule);
    }

    #[must_use]
    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.get(id)
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values()
    }

    #[must_use]
    pub fn with_default_rules() -> Self {
        let mut registry = Self::new();

        registry.register(Rule::Ast(
            Box::<snake_case_variables::SnakeCaseVariables>::default(),
        ));
        registry.register(Rule::Ast(
            Box::<kebab_case_commands::KebabCaseCommands>::default(),
        ));
        registry.register(Rule::Regex(Box::<
            screaming_snake_constants::ScreamingSnakeConstants,
        >::default()));
        registry.register(Rule::Regex(Box::new(
            unnecessary_variable_before_return::UnnecessaryVariableBeforeReturn::new(),
        )));
        registry.register(Rule::Ast(Box::new(prefer_is_not_empty::PreferIsNotEmpty)));
        registry.register(Rule::Regex(Box::new(
            discourage_bare_ignore::DiscouragedBareIgnore::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            discourage_underscore_commands::DiscourageUnderscoreCommands::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            completion_function_naming::CompletionFunctionNaming::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            multiline_formatting::MultilineFormatting,
        )));
        registry.register(Rule::Regex(Box::new(no_trailing_spaces::NoTrailingSpaces)));
        registry.register(Rule::Ast(Box::<brace_spacing::BraceSpacing>::default()));
        registry.register(Rule::Ast(Box::<pipe_spacing::PipeSpacing>::default()));
        registry.register(Rule::Ast(Box::<
            prefer_compound_assignment::PreferCompoundAssignment,
        >::default()));
        registry.register(Rule::Ast(Box::new(unnecessary_mut::UnnecessaryMut::new())));
        registry.register(Rule::Ast(Box::new(omit_list_commas::OmitListCommas)));
        registry.register(Rule::Regex(Box::new(
            prefer_error_make::PreferErrorMake::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            avoid_mutable_accumulation::AvoidMutableAccumulation,
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_range_iteration::PreferRangeIteration::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_parse_command::PreferParseCommand::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            consistent_error_handling::ConsistentErrorHandling::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_match_over_if_chain::PreferMatchOverIfChain::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_each_over_for::PreferEachOverFor::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            descriptive_error_messages::DescriptiveErrorMessages::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_builtin_commands::AvoidExternalFileTools::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_builtin_text_transforms::AvoidExternalTextTools::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_builtin_system_commands::AvoidExternalSystemTools::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_where_over_each_if::PreferWhereOverEachIf,
        )));
        registry.register(Rule::Regex(Box::new(
            prefer_lines_over_split::PreferLinesOverSplit::new(),
        )));
        registry.register(Rule::Ast(Box::new(
            prefer_parse_over_each_split::PreferParseOverEachSplit,
        )));
        registry.register(Rule::Regex(Box::new(
            missing_command_docs::MissingCommandDocs::new(),
        )));
        registry.register(Rule::Regex(Box::new(
            exported_function_docs::ExportedFunctionDocs,
        )));
        registry.register(Rule::Ast(Box::new(
            missing_type_annotation::MissingTypeAnnotation::new(),
        )));

        registry
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
