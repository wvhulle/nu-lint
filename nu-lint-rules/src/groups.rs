use nu_lint_analysis::rule::Group;

const CATCH_RUNTIME_FAILURES: Group = Group {
    name: "catch-runtime-failures",
    description: "Stop scripts that parse fine from failing or misbehaving at runtime.",
    rules: &[
        super::add_hat_external_commands::RULE,
        super::fragile_last_exit_code::RULE,
        super::check_complete_exit_code::RULE,
        super::serialize_data_for_external::RULE,
        super::unescaped_interpolation::RULE,
        super::exit_only_in_main::RULE,
        super::check_typed_flag_before_use::RULE,
        super::non_final_failure_check::RULE,
        super::error_make::error_make_for_non_fatal::RULE,
        super::try_instead_of_do::RULE,
        super::unsafe_dynamic_record_access::RULE,
        super::missing_stdin_in_shebang::RULE,
        super::dynamic_script_import::RULE,
        super::self_import::RULE,
        super::catch_builtin_error_try::RULE,
        super::default_empty_string_masks_missing::RULE,
        super::unchecked_cell_path_index::RULE,
        super::unchecked_get_index::RULE,
        super::unhandled_external_error::RULE,
        super::spread_list_to_external::RULE,
        super::glob_may_drop_quotes::RULE,
        super::require_main_with_stdin::RULE,
    ],
};

const ISOLATE_SIDE_EFFECTS: Group = Group {
    name: "isolate-side-effects",
    description: "Keep I/O, printing and dangerous file operations out of data-returning code.",
    rules: &[
        super::dangerous_file_operations::RULE,
        super::errors_to_stderr::RULE,
        super::side_effects::dont_mix_different_effects::RULE,
        super::side_effects::print_and_return_data::RULE,
        super::side_effects::each_nothing_to_for_loop::RULE,
        super::side_effects::silence_stderr_data::RULE,
    ],
};

const ANNOTATE_TYPES: Group = Group {
    name: "annotate-types",
    description: "Add or tighten type annotations in signatures.",
    rules: &[
        super::external_script_as_argument::RULE,
        super::nothing_outside_signature::RULE,
        super::typing::add_type_hints_arguments::RULE,
        super::filesystem::string_param_as_path::RULE,
        super::typing::missing_output_type::RULE,
        super::typing::missing_in_type::RULE,
    ],
};

const ADOPT_NU_SYNTAX: Group = Group {
    name: "adopt-nu-syntax",
    description: "Replace verbose code with Nu operators, spread, assignment forms and the \
                  standard library.",
    rules: &[
        super::not_is_empty_to_is_not_empty::RULE,
        super::columns_in_to_has::RULE,
        super::columns_not_in_to_not_has::RULE,
        super::dispatch_with_subcommands::RULE,
        super::get_optional_to_has::RULE,
        super::get_optional_to_not_has::RULE,
        super::hardcoded_math_constants::RULE,
        super::merge_get_cell_path::RULE,
        super::merge_multiline_print::RULE,
        super::source_to_use::RULE,
        super::compound_assignment::RULE,
        super::contains_to_regex_op::RULE,
        super::ansi_over_escape_codes::RULE,
        super::append_to_concat_assign::RULE,
        super::custom_log_command::RULE,
        super::chained_append::RULE,
        super::record_assignments::USE_RECORD_SPREAD,
        super::record_assignments::USE_LOAD_ENV,
        super::remove_hat_not_builtin::RULE,
        super::division_to_format_duration::RULE,
        super::ignore_over_dev_null::RULE,
        super::redundant_echo::RULE,
        super::if_null_to_default::RULE,
        super::list_param_to_variadic::RULE,
    ],
};

const PIPE_INSTEAD_OF_LOOP: Group = Group {
    name: "pipe-instead-of-loop",
    description: "Turn loops, counters, slices and manual filters into pipeline commands.",
    rules: &[
        super::filtering::each_if_to_where::RULE,
        super::filtering::for_filter_to_where::RULE,
        super::filtering::omit_it_in_row_condition::RULE,
        super::filtering::slice_to_drop::RULE,
        super::filtering::slice_to_last::RULE,
        super::filtering::slice_to_skip::RULE,
        super::filtering::slice_to_take::RULE,
        super::filtering::uniq_to_set_operation::RULE,
        super::filtering::where_closure_drop_parameter::RULE,
        super::remove_redundant_in::RULE,
        super::range_for_iteration::loop_counter::RULE,
        super::range_for_iteration::while_counter::RULE,
        super::transpose_items::RULE,
        super::unnecessary_accumulate::RULE,
    ],
};

const PARSE_STRUCTURED_DATA: Group = Group {
    name: "parse-structured-data",
    description: "Get structured data from text and files with `parse`, `lines` and `open`.",
    rules: &[
        super::parsing::lines_instead_of_split::RULE,
        super::never_space_split::RULE,
        super::parsing::lines_each_to_parse::RULE,
        super::parsing::simplify_regex_parse::RULE,
        super::parsing::split_row_get_multistatement::RULE,
        super::parsing::split_first_to_parse::RULE,
        super::parsing::split_row_get_inline::RULE,
        super::parsing::split_row_space_to_split_words::RULE,
        super::filesystem::from_after_parsed_open::RULE,
        super::filesystem::open_raw_from_to_open::RULE,
    ],
};

const REPLACE_EXTERNAL_TOOLS: Group = Group {
    name: "replace-external-tools",
    description: "Swap external tools such as `grep`, `sed` and `jq` for equivalent built-ins.",
    rules: &[
        super::external_tools::cat_to_open::RULE,
        super::external_tools::curl_wget_to_http::RULE,
        super::external_tools::cd_to_builtin::RULE,
        super::external_tools::date_to_builtin::RULE,
        super::external_tools::find_to_glob::RULE,
        super::external_tools::grep_to_where::RULE,
        super::external_tools::head_tail_to_first_last::RULE,
        super::external_tools::jq_to_nu_pipeline::RULE,
        super::external_tools::ls_to_builtin::RULE,
        super::external_tools::read_to_input::RULE,
        super::external_tools::sed_to_str_replace::RULE,
        super::external_tools::sort_to_builtin::RULE,
        super::external_tools::system_info_to_sys::RULE,
        super::external_tools::tac_to_reverse::RULE,
        super::external_tools::wc_to_length::RULE,
    ],
};

const STREAM_IN_PROCESS: Group = Group {
    name: "stream-in-process",
    description: "Avoid subprocesses and buffering so data streams inside one Nu process.",
    rules: &[
        super::redundant_nu_subprocess::RULE,
        super::nu_script_to_run::RULE,
        super::positional_to_pipeline::RULE,
        super::chained_str_transform::RULE,
        super::streaming_hidden_by_complete::RULE,
    ],
};

const REMOVE_DEAD_CODE: Group = Group {
    name: "remove-dead-code",
    description: "Delete unused or redundant code.",
    rules: &[
        super::assign_then_return::RULE,
        super::do_not_compare_booleans::RULE,
        super::redundant_ignore::RULE,
        super::unnecessary_mut::RULE,
        super::unused_helper_functions::RULE,
        super::unused_parameter::RULE,
        super::unused_variable::RULE,
        super::script_export_main::RULE,
    ],
};

const REDUCE_COMPLEXITY: Group = Group {
    name: "reduce-complexity",
    description: "Flatten nesting, shorten bodies and simplify signatures and calls.",
    rules: &[
        super::collapsible_if::RULE,
        super::forbid_excessive_nesting::RULE,
        super::max_function_body_length::RULE,
        super::if_else_chain_to_match::RULE,
        super::max_positional_params::RULE,
        super::explicit_long_flags::RULE,
        super::single_call_command::RULE,
    ],
};

const FORMAT_WHITESPACE: Group = Group {
    name: "format-whitespace",
    description: "Fix spacing, line width and quoting according to the style guide.",
    rules: &[
        super::spacing::block_brace_spacing::RULE,
        super::spacing::closure_brace_pipe_spacing::RULE,
        super::spacing::closure_pipe_body_spacing::RULE,
        super::spacing::no_trailing_spaces::RULE,
        super::spacing::omit_list_commas::RULE,
        super::spacing::pipe_spacing::RULE,
        super::spacing::record_brace_spacing::RULE,
        super::spacing::reflow_wide_pipelines::RULE,
        super::spacing::reflow_wide_lists::RULE,
        super::spacing::wrap_wide_records::RULE,
        super::string_may_be_bare::RULE,
    ],
};

const FOLLOW_NAMING_CONVENTIONS: Group = Group {
    name: "follow-naming-conventions",
    description: "Apply the casing of the official style guide.",
    rules: &[
        super::naming::kebab_case_commands::RULE,
        super::naming::screaming_snake_constants::RULE,
        super::naming::snake_case_variables::RULE,
    ],
};

const DOCUMENT_INTERFACES: Group = Group {
    name: "document-interfaces",
    description: "Add docs to commands and context to `error make`.",
    rules: &[
        super::documentation::add_doc_comment_exported_fn::RULE,
        super::documentation::descriptive_error_messages::RULE,
        super::error_make::add_label_to_error::RULE,
        super::error_make::add_help_to_error::RULE,
        super::error_make::add_span_to_label::RULE,
        super::error_make::add_url_to_error::RULE,
        super::documentation::main_positional_args_docs::RULE,
        super::documentation::main_named_args_docs::RULE,
    ],
};

const FORWARD_PARSER_DIAGNOSTICS: Group = Group {
    name: "forward-parser-diagnostics",
    description: "Report errors and unhandled deprecations from nu-parser.",
    rules: &[
        super::upstream::nu_deprecated::RULE,
        super::upstream::nu_parse_error::RULE,
    ],
};

const MIGRATE_DEPRECATED_SYNTAX: Group = Group {
    name: "migrate-deprecated-syntax",
    description: "Rewrite syntax that nu-parser deprecated.",
    rules: &[
        super::migrations::filter_to_where::RULE,
        super::migrations::str_case_full_names::RULE,
        super::migrations::ignore_errors_to_optional::RULE,
        super::migrations::drop_get_sensitive::RULE,
        super::migrations::legacy_completer_inputs::RULE,
    ],
};

pub const ALL_GROUPS: &[Group] = &[
    CATCH_RUNTIME_FAILURES,
    ISOLATE_SIDE_EFFECTS,
    ANNOTATE_TYPES,
    ADOPT_NU_SYNTAX,
    PIPE_INSTEAD_OF_LOOP,
    PARSE_STRUCTURED_DATA,
    REPLACE_EXTERNAL_TOOLS,
    STREAM_IN_PROCESS,
    REMOVE_DEAD_CODE,
    REDUCE_COMPLEXITY,
    FORMAT_WHITESPACE,
    FOLLOW_NAMING_CONVENTIONS,
    DOCUMENT_INTERFACES,
    FORWARD_PARSER_DIAGNOSTICS,
    MIGRATE_DEPRECATED_SYNTAX,
];

#[must_use]
pub fn group_for_rule(rule_id: &str) -> Option<&'static str> {
    ALL_GROUPS
        .iter()
        .find(|group| group.rules.iter().any(|rule| rule.id() == rule_id))
        .map(|group| group.name)
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write, fs};

    use super::{ALL_GROUPS, Group};
    use crate::USED_RULES;

    fn readme_markdown() -> String {
        ALL_GROUPS.iter().map(group_details_markdown).collect()
    }

    fn group_details_markdown(group: &Group) -> String {
        let rule_lines = group.rules.iter().fold(String::new(), |mut lines, rule| {
            let auto_fix_suffix = if rule.has_auto_fix() {
                " (auto-fix)"
            } else {
                ""
            };
            writeln!(
                lines,
                "- `{}`{auto_fix_suffix}: {}",
                rule.id(),
                rule.short_description()
            )
            .unwrap();
            lines
        });
        format!(
            "<details>\n<summary><code>{}</code> ({} rules): \
             {}</summary>\n\n{rule_lines}\n</details>\n\n",
            group.name,
            group.rules.len(),
            group.description
        )
    }

    const README_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../README.md");
    const START_MARKER: &str = "<!-- start-rule-groups -->\n";
    const END_MARKER: &str = "<!-- end-rule-groups -->";

    #[test]
    fn readme_rule_list_is_up_to_date() {
        let readme = fs::read_to_string(README_PATH).unwrap();
        let (before_list, rest) = readme.split_once(START_MARKER).unwrap();
        let (_, after_list) = rest.split_once(END_MARKER).unwrap();
        let regenerated = format!(
            "{before_list}{START_MARKER}{}{END_MARKER}{after_list}",
            readme_markdown()
        );
        if regenerated != readme {
            fs::write(README_PATH, regenerated).unwrap();
            panic!("README.md rule list was outdated and has been regenerated; commit the change");
        }
    }

    fn group_count_of(rule_id: &str) -> usize {
        ALL_GROUPS
            .iter()
            .filter(|group| group.rules.iter().any(|rule| rule.id() == rule_id))
            .count()
    }

    #[test]
    fn every_used_rule_is_in_exactly_one_group() {
        let misgrouped_rules: Vec<_> = USED_RULES
            .iter()
            .map(|rule| (rule.id(), group_count_of(rule.id())))
            .filter(|(_, group_count)| *group_count != 1)
            .collect();
        assert!(
            misgrouped_rules.is_empty(),
            "rules must be in exactly one group, found (rule, group count): {misgrouped_rules:?}"
        );
    }

    #[test]
    fn every_grouped_rule_is_used() {
        let unused_grouped_rules: Vec<_> = ALL_GROUPS
            .iter()
            .flat_map(|group| group.rules)
            .map(|rule| rule.id())
            .filter(|rule_id| !USED_RULES.iter().any(|rule| rule.id() == *rule_id))
            .collect();
        assert!(
            unused_grouped_rules.is_empty(),
            "grouped rules missing from USED_RULES: {unused_grouped_rules:?}"
        );
    }

    #[test]
    fn group_names_are_unique() {
        let duplicated_names: Vec<_> = ALL_GROUPS
            .iter()
            .map(|group| group.name)
            .filter(|name| {
                ALL_GROUPS
                    .iter()
                    .filter(|group| group.name == *name)
                    .count()
                    > 1
            })
            .collect();
        assert!(
            duplicated_names.is_empty(),
            "duplicated group names: {duplicated_names:?}"
        );
    }
}
