// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::fmt::Write;
use {
    crate::{
        enum_only_single_field_unnamed_variants,
        errors::Errors,
        parse_attrs::{check_enum_type_attrs, Description, FieldKind, TypeAttrs, VariantAttrs},
        FieldAttrs, Optionality, StructField,
    },
    argh_shared::INDENT,
    proc_macro2::{Span, TokenStream},
    quote::{quote, quote_spanned, ToTokens},
};

const SECTION_SEPARATOR: &str = "\n\n";

/// Internal data used to create the HelpPositionalInfo during the
/// code generation.
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

/// Internal data used to create HelpFlagInfo during the code
/// generation.
struct FlagInfo {
    short: Option<char>,
    long: String,
    description: String,
    optionality: argh_shared::HelpOptionality,
    kind: FlagKind,
}

enum FlagKind {
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
            FlagKind::Switch
        } else {
            let arg_name = if let Some(arg_name) = &field.attrs.arg_name {
                arg_name.value()
            } else {
                long.trim_start_matches("--").to_owned()
            };
            FlagKind::Option { arg_name }
        };

        Self { short, long, description, optionality: to_help_optional(&field.optionality), kind }
    }

    fn as_help_info(&self) -> argh_shared::HelpFlagInfo {
        let kind = match &self.kind {
            FlagKind::Switch => argh_shared::HelpFieldKind::Switch,
            FlagKind::Option { arg_name } => {
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

/// Convert optionality to the HelpOptionality type.
fn to_help_optional(optionality: &Optionality) -> argh_shared::HelpOptionality {
    match optionality {
        // None means it is required
        Optionality::None => argh_shared::HelpOptionality::None,
        // fields with default values are optional.
        Optionality::Defaulted(_) => argh_shared::HelpOptionality::Optional,
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

    // Convert the internal PositionalInfo into HelpPositionalInfo which is used as in the generated code.
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
                    .iter()
                    .copied()
                    .chain(
                        <#subcommand_ty as argh::SubCommands>::dynamic_commands()
                            .iter()
                            .copied())
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
            write!(format_lit, "{} {}", code, text.value()).unwrap();
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

    let info = argh_shared::CommandInfo { name: &name, description };
    argh_shared::write_description(out, &info);
}

pub(crate) fn impl_derive_help(input: &syn::DeriveInput) -> TokenStream {
    let errors = &Errors::default();
    if !input.generics.params.is_empty() {
        errors.err(
            &input.generics,
            "`#![derive(Help)]` cannot be applied to types with generic parameters",
        );
    }
    let type_attrs = &TypeAttrs::parse(errors, input);
    let mut output_tokens = match &input.data {
        syn::Data::Struct(ds) => impl_help_from_args_struct(errors, &input.ident, type_attrs, ds),
        syn::Data::Enum(de) => impl_help_from_args_enum(errors, &input.ident, type_attrs, de),
        syn::Data::Union(_) => {
            errors.err(input, "`#[derive(Help)]` cannot be applied to unions");
            TokenStream::new()
        }
    };
    errors.to_tokens(&mut output_tokens);
    output_tokens
}

fn impl_help<'a>(
    errors: &Errors,
    type_attrs: &TypeAttrs,
    fields: &'a [StructField<'a>],
) -> TokenStream {
    let mut subcommands_iter =
        fields.iter().filter(|field| field.kind == FieldKind::SubCommand).fuse();

    let subcommand: Option<&StructField<'_>> = subcommands_iter.next();
    for dup_subcommand in subcommands_iter {
        errors.duplicate_attrs("subcommand", subcommand.unwrap().field, dup_subcommand.field);
    }

    let impl_span = Span::call_site();

    let mut positionals = vec![];
    let mut flags = vec![];

    for field in fields {
        let optionality = match field.optionality {
            Optionality::None => quote! { argh::HelpOptionality::None },
            Optionality::Defaulted(_) => quote! { argh::HelpOptionality::Optional },
            Optionality::Optional => quote! { argh::HelpOptionality::Optional },
            Optionality::Repeating => quote! { argh::HelpOptionality::Repeating },
        };

        match field.kind {
            FieldKind::Positional => {
                let name = field.arg_name();

                let description = if let Some(desc) = &field.attrs.description {
                    desc.content.value().trim().to_owned()
                } else {
                    String::new()
                };

                positionals.push(quote! {
                    argh::HelpPositionalInfo {
                        name: #name,
                        description: #description,
                        optionality: #optionality,
                    }
                });
            }
            FieldKind::Switch | FieldKind::Option => {
                let short = if let Some(short) = &field.attrs.short {
                    quote! { Some(#short) }
                } else {
                    quote! { None }
                };

                let long = field.long_name.as_ref().expect("missing long name for option");

                let description = require_description(
                    errors,
                    field.name.span(),
                    &field.attrs.description,
                    "field",
                );

                let kind = if field.kind == FieldKind::Switch {
                    quote! {
                        argh::HelpFieldKind::Switch
                    }
                } else {
                    let arg_name = if let Some(arg_name) = &field.attrs.arg_name {
                        quote! { #arg_name }
                    } else {
                        let arg_name = long.trim_start_matches("--");
                        quote! { #arg_name }
                    };

                    quote! {
                        argh::HelpFieldKind::Option {
                            arg_name: #arg_name,
                        }
                    }
                };

                flags.push(quote! {
                    argh::HelpFlagInfo {
                        short: #short,
                        long: #long,
                        description: #description,
                        optionality: #optionality,
                        kind: #kind,
                    }
                });
            }
            FieldKind::SubCommand => {}
        }
    }

    let subcommand = if let Some(subcommand) = subcommand {
        let subcommand_ty = subcommand.ty_without_wrapper;
        quote! { Some(<#subcommand_ty as argh::HelpSubCommands>::HELP_INFO) }
    } else {
        quote! { None }
    };

    let description =
        require_description(errors, Span::call_site(), &type_attrs.description, "type");
    let examples = type_attrs.examples.iter().map(|e| quote! { #e });
    let notes = type_attrs.notes.iter().map(|e| quote! { #e });

    let error_codes = type_attrs.error_codes.iter().map(|(code, text)| {
        quote! { (#code, #text) }
    });

    quote_spanned! { impl_span =>
        argh::HelpInfo {
            description: #description,
            examples: &[#( #examples, )*],
            notes: &[#( #notes, )*],
            positionals: &[#( #positionals, )*],
            flags: &[#( #flags, )*],
            subcommand: #subcommand,
            error_codes: &[#( #error_codes, )*],
        }
    }
}

fn impl_help_from_args_struct(
    errors: &Errors,
    name: &syn::Ident,
    type_attrs: &TypeAttrs,
    ds: &syn::DataStruct,
) -> TokenStream {
    let fields = match &ds.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(_) => {
            errors.err(
                &ds.struct_token,
                "`#![derive(FromArgs)]` is not currently supported on tuple structs",
            );
            return TokenStream::new();
        }
        syn::Fields::Unit => {
            errors.err(&ds.struct_token, "#![derive(FromArgs)]` cannot be applied to unit structs");
            return TokenStream::new();
        }
    };

    let fields: Vec<_> = fields
        .named
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttrs::parse(errors, field);
            StructField::new(errors, field, attrs)
        })
        .collect();

    let impl_span = Span::call_site();

    let help_info = impl_help(errors, type_attrs, &fields);

    let top_or_sub_cmd_help_impl = top_or_sub_cmd_help_impl(errors, name, type_attrs);

    let trait_impl = quote_spanned! { impl_span =>
        #[automatically_derived]
        impl argh::Help for #name {
            const HELP_INFO: &'static argh::HelpInfo = &#help_info;
        }

        #top_or_sub_cmd_help_impl
    };

    trait_impl
}

fn impl_help_from_args_enum(
    errors: &Errors,
    name: &syn::Ident,
    type_attrs: &TypeAttrs,
    de: &syn::DataEnum,
) -> TokenStream {
    check_enum_type_attrs(errors, type_attrs, &de.enum_token.span);

    // An enum variant like `<name>(<ty>)`
    struct SubCommandVariant<'a> {
        ty: &'a syn::Type,
    }

    let mut dynamic_type_and_variant = None;

    let variants: Vec<SubCommandVariant<'_>> = de
        .variants
        .iter()
        .filter_map(|variant| {
            let name = &variant.ident;
            let ty = enum_only_single_field_unnamed_variants(errors, &variant.fields)?;
            if VariantAttrs::parse(errors, variant).is_dynamic.is_some() {
                if dynamic_type_and_variant.is_some() {
                    errors.err(variant, "Only one variant can have the `dynamic` attribute");
                }
                dynamic_type_and_variant = Some((ty, name));
                None
            } else {
                Some(SubCommandVariant { ty })
            }
        })
        .collect();

    let variant_ty = variants.iter().map(|x| x.ty).collect::<Vec<_>>();

    let dynamic_commands_help = dynamic_type_and_variant.as_ref().map(|(dynamic_type, _)| {
        quote! {
            fn dynamic_commands_help() -> &'static [&'static argh::HelpSubCommandsInfo] {
                <#dynamic_type as argh::DynamicHelpSubCommand>::commands()
            }
        }
    });

    quote! {
        impl argh::HelpSubCommands for #name {
            const HELP_INFO: &'static argh::HelpSubCommandsInfo = &argh::HelpSubCommandsInfo {
                optional: false,
                commands: &[#(
                    <#variant_ty as argh::HelpSubCommand>::HELP_INFO,
                )*],
            };
            #dynamic_commands_help
        }
    }
}

fn top_or_sub_cmd_help_impl(
    errors: &Errors,
    name: &syn::Ident,
    type_attrs: &TypeAttrs,
) -> TokenStream {
    if type_attrs.is_subcommand.is_some() {
        let empty_str = syn::LitStr::new("", Span::call_site());
        let subcommand_name = type_attrs.name.as_ref().unwrap_or_else(|| {
            errors.err(name, "`#[argh(name = \"...\")]` attribute is required for subcommands");
            &empty_str
        });
        quote! {
            #[automatically_derived]
            impl argh::HelpSubCommand for #name {
                const HELP_INFO: &'static argh::HelpSubCommandInfo = &argh::HelpSubCommandInfo {
                    name: #subcommand_name,
                    info: <#name as argh::Help>::HELP_INFO,
                };
            }
        }
    } else {
        quote! {}
    }
}
