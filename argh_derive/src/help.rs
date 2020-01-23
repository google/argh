// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use {
    crate::{
        errors::Errors,
        parse_attrs::{Description, FieldKind, TypeAttrs},
        Optionality, StructField,
    },
    argh_shared::INDENT,
    proc_macro2::{Span, TokenStream},
    quote::quote,
};

const SECTION_SEPARATOR: &str = "\n\n";

/// Returns a `TokenStream` generating a `String` help message.
///
/// Note: `fields` entries with `is_subcommand.is_some()` will be ignored
/// in favor of the `subcommand` argument.
pub(crate) fn help(
    errors: &Errors,
    cmd_name_str_array_ident: syn::Ident,
    ty_attrs: &TypeAttrs,
    fields: &[StructField<'_>],
    subcommand: Option<&StructField<'_>>,
) -> TokenStream {
    let mut format_lit = "Usage: {command_name}".to_string();

    let positional = fields.iter().filter(|f| f.kind == FieldKind::Positional);
    for arg in positional {
        format_lit.push(' ');
        positional_usage(&mut format_lit, arg);
    }

    let options = fields.iter().filter(|f| f.long_name.is_some());
    for option in options.clone() {
        format_lit.push(' ');
        option_usage(&mut format_lit, option);
    }

    if let Some(subcommand) = subcommand {
        format_lit.push(' ');
        if !subcommand.optionality.is_required() {
            format_lit.push('[');
        }
        format_lit.push_str("<command>");
        if !subcommand.optionality.is_required() {
            format_lit.push(']');
        }
        format_lit.push_str(" [<args>]");
    }

    format_lit.push_str(SECTION_SEPARATOR);

    let description = require_description(errors, Span::call_site(), &ty_attrs.description, "type");
    format_lit.push_str(&description);

    format_lit.push_str(SECTION_SEPARATOR);
    format_lit.push_str("Options:");
    for option in options {
        option_description(errors, &mut format_lit, option);
    }
    // Also include "help"
    option_description_format(&mut format_lit, None, "--help", "display usage information");

    let subcommand_calculation;
    let subcommand_format_arg;
    if let Some(subcommand) = subcommand {
        format_lit.push_str(SECTION_SEPARATOR);
        format_lit.push_str("Commands:{subcommands}");
        let subcommand_ty = subcommand.ty_without_wrapper;
        subcommand_format_arg = quote! { subcommands = subcommands };
        subcommand_calculation = quote! {
            let subcommands = argh::print_subcommands(
                <#subcommand_ty as argh::SubCommands>::COMMANDS
            );
        };
    } else {
        subcommand_calculation = TokenStream::new();
        subcommand_format_arg = TokenStream::new()
    }

    lits_section(&mut format_lit, "Examples:", &ty_attrs.examples);

    lits_section(&mut format_lit, "Notes:", &ty_attrs.notes);

    if ty_attrs.error_codes.len() != 0 {
        format_lit.push_str(SECTION_SEPARATOR);
        format_lit.push_str("Error codes:");
        for (code, text) in &ty_attrs.error_codes {
            format_lit.push('\n');
            format_lit.push_str(INDENT);
            format_lit.push_str(&format!("{} {}", code, text.value()));
        }
    }

    format_lit.push_str("\n");

    quote! { {
        #subcommand_calculation
        format!(#format_lit, command_name = #cmd_name_str_array_ident.join(" "), #subcommand_format_arg)
    } }
}

/// A section composed of exactly just the literals provided to the program.
fn lits_section(out: &mut String, heading: &str, lits: &[syn::LitStr]) {
    if lits.len() != 0 {
        out.push_str(SECTION_SEPARATOR);
        out.push_str(heading);
        for lit in lits {
            let value = lit.value();
            for line in value.split('\n') {
                out.push('\n');
                out.push_str(INDENT);
                out.push_str(line);
            }
        }
    }
}

/// Add positional arguments like `[<foo>...]` to a help format string.
fn positional_usage(out: &mut String, field: &StructField<'_>) {
    if !field.optionality.is_required() {
        out.push('[');
    }
    out.push('<');
    out.push_str(&field.name.to_string());
    if field.optionality == Optionality::Repeating {
        out.push_str("...");
    }
    out.push('>');
    if !field.optionality.is_required() {
        out.push(']');
    }
}

/// Add options like `[-f <foo>]` to a help format string.
/// This function must only be called on options (things with `long_name.is_some()`)
fn option_usage(out: &mut String, field: &StructField<'_>) {
    // bookend with `[` and `]` if optional
    if !field.optionality.is_required() {
        out.push('[');
    }

    let long_name = field.long_name.as_ref().expect("missing long name for option");
    if let Some(short) = field.attrs.short.as_ref() {
        out.push('-');
        out.push(short.value());
    } else {
        out.push_str(long_name);
    }

    match field.kind {
        FieldKind::SubCommand | FieldKind::Positional => unreachable!(), // don't have long_name
        FieldKind::Switch => {}
        FieldKind::Option => {
            out.push_str(" <");
            out.push_str(long_name.trim_start_matches("--"));
            out.push('>');
        }
    }

    if !field.optionality.is_required() {
        out.push(']');
    }
}

// TODO(cramertj) make it so this is only called at least once per object so
// as to avoid creating multiple errors.
pub fn require_description(
    errors: &Errors,
    err_span: Span,
    desc: &Option<Description>,
    kind: &str, // the thing being described ("type" or "field"),
) -> String {
    desc.as_ref().map(|d| d.content.value().trim().to_owned()).unwrap_or_else(|| {
        errors.err_span(
            err_span,
            &format!(
                "#[derive(FromArgs)] {} with no description.
Add a doc comment or an `#[argh(description = \"...\")]` attribute.",
                kind
            ),
        );
        "".to_string()
    })
}

/// Describes an option like this:
///  -f, --force       force, ignore minor errors. This description
///                    is so long that it wraps to the next line.
fn option_description(errors: &Errors, out: &mut String, field: &StructField<'_>) {
    let short = field.attrs.short.as_ref().map(|s| s.value());
    let long_with_leading_dashes = field.long_name.as_ref().expect("missing long name for option");
    let description =
        require_description(errors, field.name.span(), &field.attrs.description, "field");

    option_description_format(out, short, long_with_leading_dashes, &description)
}

fn option_description_format(
    out: &mut String,
    short: Option<char>,
    long_with_leading_dashes: &str,
    description: &str,
) {
    let mut name = String::new();
    if let Some(short) = short {
        name.push('-');
        name.push(short);
        name.push_str(", ");
    }
    name.push_str(long_with_leading_dashes);

    let info = argh_shared::CommandInfo { name: &*name, description };
    argh_shared::write_description(out, &info);
}
