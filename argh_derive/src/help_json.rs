// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use {
    crate::{
        errors::Errors,
        help::{build_usage_command_line, require_description, HELP_DESCRIPTION, HELP_FLAG},
        parse_attrs::{FieldKind, TypeAttrs},
        Optionality, StructField,
    },
    proc_macro2::{Span, TokenStream},
    quote::quote,
};

struct OptionHelp {
    short: String,
    long: String,
    description: String,
    arg_name: String,
    optionality: String,
}

struct PositionalHelp {
    name: String,
    description: String,
    optionality: String,
}
struct HelpJSON {
    usage: String,
    description: String,
    positional_args: Vec<PositionalHelp>,
    options: Vec<OptionHelp>,
    examples: String,
    notes: String,
    error_codes: Vec<PositionalHelp>,
}

fn option_elements_json(options: &[OptionHelp]) -> String {
    let mut retval = String::from("");
    for opt in options {
        if !retval.is_empty() {
            retval.push_str(",\n");
        }
        retval.push_str(&format!(
            "{{\"short\": \"{}\", \"long\": \"{}\", \"description\": \"{}\", \"arg_name\": \"{}\", \"optionality\": \"{}\"}}",
            opt.short,
            opt.long,
            escape_json(&opt.description),
            opt.arg_name,
            escape_json(&opt.optionality)
        ));
    }
    retval
}
fn help_elements_json(elements: &[PositionalHelp], skip_optionality: bool) -> String {
    let mut retval = String::from("");
    for pos in elements {
        if !retval.is_empty() {
            retval.push_str(",\n");
        }
        retval.push_str(&format!(
            "{{\"name\": \"{}\", \"description\": \"{}\"",
            pos.name,
            escape_json(&pos.description),
        ));
        if !skip_optionality {
            retval.push_str(&format!(", \"optionality\": \"{}\"", escape_json(&pos.optionality)));
        }
        retval.push('}');
    }
    retval
}

/// Returns a `TokenStream` generating a `String` help message containing JSON.
///
/// Note: `fields` entries with `is_subcommand.is_some()` will be ignored
/// in favor of the `subcommand` argument.
pub(crate) fn help_json(
    errors: &Errors,
    cmd_name_str_array_ident: &syn::Ident,
    ty_attrs: &TypeAttrs,
    fields: &[StructField<'_>],
) -> TokenStream {
    let mut usage_format_pattern = "{command_name}".to_string();

    let mut subcommands_iter =
        fields.iter().filter(|field| field.kind == FieldKind::SubCommand).fuse();
    let subcommand: Option<&StructField<'_>> = subcommands_iter.next();

    build_usage_command_line(&mut usage_format_pattern, fields, subcommand);

    let mut help_obj = HelpJSON {
        usage: String::from(""),
        description: String::from(""),
        positional_args: vec![],
        options: vec![],
        examples: String::from(""),
        notes: String::from(""),
        error_codes: vec![],
    };

    // Add positional args to the help object.
    let positional = fields.iter().filter(|f| f.kind == FieldKind::Positional);
    for arg in positional {
        let mut description = String::from("");
        if let Some(desc) = &arg.attrs.description {
            description = desc.content.value().trim().to_owned();
        }
        let optionality = match &arg.optionality {
            Optionality::None => String::from("required"),
            Optionality::Optional => String::from("optional"),
            Optionality::Repeating => String::from("repeating"),
            Optionality::Defaulted(ts) => ts.to_string(),
        };
        help_obj.positional_args.push(PositionalHelp {
            name: arg.arg_name(),
            description,
            optionality,
        });
    }

    // Add options to the help object.
    let options = fields.iter().filter(|f| f.long_name.is_some());
    for option in options {
        let field_kind = match &option.attrs.field_type {
            Some(field_type) => field_type.kind,
            _ => unreachable!("Field type not set"),
        };
        let short = match option.attrs.short.as_ref().map(|s| s.value()) {
            Some(c) => String::from(c),
            None => String::from(""),
        };
        let long_with_leading_dashes =
            option.long_name.as_ref().expect("missing long name for option");
        let description =
            require_description(errors, option.name.span(), &option.attrs.description, "field");

        let arg_name = match field_kind {
            FieldKind::Option | FieldKind::Positional => match &option.attrs.arg_name {
                Some(_) => option.arg_name(),
                // None if field_kind != FieldKind::Switch => option.name.to_string(),
                None => option.name.to_string(),
            },
            FieldKind::Switch | FieldKind::SubCommand => String::from(""),
        };

        let optionality = match &option.optionality {
            Optionality::None => String::from("required"),
            Optionality::Optional => String::from("optional"),
            Optionality::Repeating => String::from("repeating"),
            Optionality::Defaulted(ts) => ts.to_string(),
        };
        help_obj.options.push(OptionHelp {
            short,
            long: long_with_leading_dashes.to_owned(),
            description,
            arg_name,
            optionality,
        });
    }
    // Also include "help"
    help_obj.options.push(OptionHelp {
        short: String::from(""),
        long: String::from(HELP_FLAG),
        description: String::from(HELP_DESCRIPTION),
        arg_name: String::from(""),
        optionality: String::from("optional"),
    });

    let subcommand_calculation;
    if let Some(subcommand) = subcommand {
        let subcommand_ty = subcommand.ty_without_wrapper;
        subcommand_calculation = quote! {
            let mut subcommands = String::from("");
            for cmd in  <#subcommand_ty as argh::SubCommands>::COMMANDS {
                if !subcommands.is_empty() {
                    subcommands.push_str(",\n");
                }
                let mut command = __cmd_name.to_owned();
                command.push(cmd.name);
                subcommands.push_str(&<#subcommand_ty as argh::FromArgs>::help_json_from_args(&command)?);
            }
        };
    } else {
        subcommand_calculation = quote! {
            let subcommands = String::from("");
        };
    }

    help_obj.usage = usage_format_pattern.clone();

    help_obj.description =
        require_description(errors, Span::call_site(), &ty_attrs.description, "type");

    let mut example: String = String::from("");
    for lit in &ty_attrs.examples {
        example.push_str(&lit.value());
    }
    help_obj.examples = example;

    let mut note: String = String::from("");
    for lit in &ty_attrs.notes {
        note.push_str(&lit.value());
    }
    help_obj.notes = note;

    if !ty_attrs.error_codes.is_empty() {
        for (code, text) in &ty_attrs.error_codes {
            help_obj.error_codes.push(PositionalHelp {
                name: code.to_string(),
                description: text.value().to_string(),
                optionality: String::from(""),
            });
        }
    }

    let help_options_json = option_elements_json(&help_obj.options);
    let help_positional_json = help_elements_json(&help_obj.positional_args, false);
    let help_error_codes_json = help_elements_json(&help_obj.error_codes, true);

    let help_description = escape_json(&help_obj.description);
    let help_examples: TokenStream;
    let help_notes: TokenStream;

    let notes_pattern = escape_json(&help_obj.notes);
    // check if we need to interpolate the string.
    if notes_pattern.contains("{command_name}") {
        help_notes = quote! {
            json_help_string.push_str(&format!(#notes_pattern,command_name = #cmd_name_str_array_ident.join(" ")));
        };
    } else {
        help_notes = quote! {
            json_help_string.push_str(#notes_pattern);
        };
    }
    let examples_pattern = escape_json(&help_obj.examples);
    if examples_pattern.contains("{command_name}") {
        help_examples = quote! {
            json_help_string.push_str(&format!(#examples_pattern,command_name = #cmd_name_str_array_ident.join(" ")));
        };
    } else {
        help_examples = quote! {
            json_help_string.push_str(#examples_pattern);
        };
    }

    let name_string: TokenStream;
    if let Some(name) = &ty_attrs.name {
        name_string = quote! { json_help_string.push_str(#name);};
    } else {
        name_string = quote! {
            json_help_string.push_str(&#cmd_name_str_array_ident.join(" "));
        };
    }

    quote! {{
        #subcommand_calculation

        // Build up the string for json. The name of the command needs to be dereferenced, so it
        // can't be done in the macro.
        let mut json_help_string = "{\n".to_string();
        json_help_string.push_str("\"name\": \"");
        #name_string;
        json_help_string.push_str("\",\n");
        let usage_value = format!(#usage_format_pattern,command_name = #cmd_name_str_array_ident.join(" "));
        json_help_string.push_str(&format!("\"usage\": \"{}\",\n",usage_value));
        json_help_string.push_str(&format!("\"description\": \"{}\",\n", #help_description));
        json_help_string.push_str(&format!("\"flags\": [{}],\n", #help_options_json));
        json_help_string.push_str(&format!("\"positional\": [{}],\n", #help_positional_json));
        json_help_string.push_str("\"examples\": \"");
        #help_examples;
        json_help_string.push_str("\",\n");
        json_help_string.push_str("\"notes\": \"");
        #help_notes;
        json_help_string.push_str("\",\n");
        json_help_string.push_str(&format!("\"error_codes\": [{}],\n", #help_error_codes_json));
        json_help_string.push_str(&format!("\"subcommands\": [{}]\n", subcommands));
        json_help_string.push_str("}\n");
        json_help_string
    }}
}

/// Escape characters in strings to be JSON compatible.
fn escape_json(value: &str) -> String {
    value.replace("\n", r#"\n"#).replace("\"", r#"\""#)
}
