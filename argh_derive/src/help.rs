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

struct PositionalInfo {
    name: String,
    description: String,
    optionality: argh_shared::HelpOptionality,
}

impl PositionalInfo {
    fn new(field: &StructField) -> Self {
        let name = field.arg_name();
        let mut description = String::from("");
        if let Some(desc) = &field.attrs.description {
            description = desc.content.value().trim().to_owned();
        }

        Self { name, description, optionality: to_help_optional(&field.optionality) }
    }

    fn as_help_info(&self) -> argh_shared::HelpPositionalInfo {
        argh_shared::HelpPositionalInfo {
            name: &self.name,
            description: &self.description,
            optionality: self.optionality,
        }
    }
}
struct FlagInfo {
    short: Option<char>,
    long: String,
    description: String,
    optionality: argh_shared::HelpOptionality,
    kind: HelpFieldKind,
}

enum HelpFieldKind {
    Switch,
    Option { arg_name: String },
}

impl FlagInfo {
    fn new(errors: &Errors, field: &StructField) -> Self {
        let short = field.attrs.short.as_ref().map(|s| s.value());

        let long = field.long_name.as_ref().expect("missing long name for option").to_owned();

        let description =
            require_description(errors, field.name.span(), &field.attrs.description, "field");

        let kind = if field.kind == FieldKind::Switch {
            HelpFieldKind::Switch
        } else {
            let arg_name = if let Some(arg_name) = &field.attrs.arg_name {
                arg_name.value()
            } else {
                long.trim_start_matches("--").to_owned()
            };
            HelpFieldKind::Option { arg_name }
        };

        Self { short, long, description, optionality: to_help_optional(&field.optionality), kind }
    }

    fn as_help_info(&self) -> argh_shared::HelpFlagInfo {
        let kind = match &self.kind {
            HelpFieldKind::Switch => argh_shared::HelpFieldKind::Switch,
            HelpFieldKind::Option { arg_name } => {
                argh_shared::HelpFieldKind::Option { arg_name: arg_name.as_str() }
            }
        };

        argh_shared::HelpFlagInfo {
            short: self.short,
            long: &self.long,
            description: &self.description,
            optionality: self.optionality,
            kind,
        }
    }
}

fn to_help_optional(optionality: &Optionality) -> argh_shared::HelpOptionality {
    match optionality {
        Optionality::None => argh_shared::HelpOptionality::None,
        Optionality::Defaulted(_) => argh_shared::HelpOptionality::None,
        Optionality::Optional => argh_shared::HelpOptionality::Optional,
        Optionality::Repeating => argh_shared::HelpOptionality::Repeating,
    }
}

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

    let positionals = fields
        .iter()
        .filter_map(|f| {
            if f.kind == FieldKind::Positional {
                Some(PositionalInfo::new(f))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let positionals = positionals.iter().map(|o| o.as_help_info()).collect::<Vec<_>>();

    let flags = fields
        .iter()
        .filter_map(|f| if f.long_name.is_some() { Some(FlagInfo::new(errors, f)) } else { None })
        .collect::<Vec<_>>();

    let flags = flags.iter().map(|o| o.as_help_info()).collect::<Vec<_>>();

    let mut has_positional = false;
    for arg in &positionals {
        has_positional = true;
        format_lit.push(' ');
        arg.help_usage(&mut format_lit);
    }

    for flag in &flags {
        format_lit.push(' ');
        flag.help_usage(&mut format_lit);
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

    if has_positional {
        format_lit.push_str(SECTION_SEPARATOR);
        format_lit.push_str("Positional Arguments:");
        for field in positionals {
            field.help_description(&mut format_lit);
        }
    }

    format_lit.push_str(SECTION_SEPARATOR);
    format_lit.push_str("Options:");
    for flag in flags {
        flag.help_description(&mut format_lit);
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

    if !ty_attrs.error_codes.is_empty() {
        format_lit.push_str(SECTION_SEPARATOR);
        format_lit.push_str("Error codes:");
        for (code, text) in &ty_attrs.error_codes {
            format_lit.push('\n');
            format_lit.push_str(INDENT);
            format_lit.push_str(&format!("{} {}", code, text.value()));
        }
    }

    format_lit.push('\n');

    quote! { {
        #subcommand_calculation
        format!(#format_lit, command_name = #cmd_name_str_array_ident.join(" "), #subcommand_format_arg)
    } }
}

/// A section composed of exactly just the literals provided to the program.
fn lits_section(out: &mut String, heading: &str, lits: &[syn::LitStr]) {
    if !lits.is_empty() {
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
