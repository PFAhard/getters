//! # Rust Getters Derive Macro Library
//! This Rust library provides a powerful and easy-to-use procedural macro `derive_getters_fn` for automatically deriving getter methods for struct fields. Designed to streamline the process of creating getters in Rust structs, this library enhances code clarity and development efficiency.
//! # Features
//! - Automatic Getter Generation: Simplifies the process of creating getters for each field in a Rust struct. This feature is particularly useful in large structs or when working with complex data structures.
//! - Customizable Through Attributes: Offers a range of attributes to customize the behavior of the generated getter methods. Attributes like `use_deref`, `use_as_ref`, `get_mut`, `skip_new`, `getter_logic`, and `skip_getter` allow developers to tailor the getters to specific requirements.
//! - Support for Various Field Types: Whether your struct has named or unnamed fields (such as in tuples), the macro can handle them efficiently, ensuring that appropriate getters are generated for each scenario.
//! - Mutable Getters: In addition to standard immutable getters, the library supports the generation of mutable getters with the get_mut attribute, providing greater flexibility.
//! - Custom Logic for Getters: The `getter_logic` attribute allows the integration of custom logic into the getter methods, offering the ability to have more complex getters beyond simple field access.
//! - Optional Constructor Generation: With the `skip_new` attribute, users can choose to generate a constructor method (new) for the struct. This is particularly useful for ensuring struct integrity upon instantiation.
//! # Usage
//! The library is designed for ease of use. After including it in your project, simply annotate your struct with `#[derive(Getters)]`, and use the provided attributes to customize the getter generation as needed.
//! # Target Audience
//! This library is ideal for Rust developers who regularly work with structs and require an efficient way to generate getters. It is especially useful in applications where data encapsulation and object-oriented patterns are prevalent.
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Ident, LitStr,
};

const USE_DEREF: &str = "use_deref";
const USE_AS_REF: &str = "use_as_ref";
const GET_MUT: &str = "get_mut";
const SKIP_NEW: &str = "skip_new";
const GETTER_LOGIC: &str = "getter_logic";
const SKIP_GETTER: &str = "skip_getter";
const RETURN_TYPE: &str = "return_type";
const COPY: &str = "copy";

/// A procedural macro to automatically derive getter methods for struct fields.
///
/// Attributes:
/// - `use_deref`: Generate a getter method that dereferences the field.
/// - `use_as_ref`: Generate a getter method using `AsRef` trait.
/// - `get_mut`: Generate a mutable getter method for the field.
/// - `skip_new`: Skip generating a `new` method for the struct.
/// - `getter_logic`: Specify custom logic for a getter method. (MUST be a function path) !!!Use with `return_type` only
/// - `skip_getter`: Do not generate a getter method for this field.
/// - `return_type`: Overrides the default return type of the getter.
/// - `copy`: Deref value in place, use for Copy types
///
/// Example:
/// ```rust
/// #[derive(Getters)]
/// struct MyStruct {
///     #[return_type = "String"]
///     field: Arc<String>,
/// }
/// ```
/// This will generate:
/// ```rust
/// pub fn field(&self) -> String {
///     self.field.clone()
/// }
/// ```
#[proc_macro_derive(
    Getters,
    attributes(
        use_deref,
        use_as_ref,
        get_mut,
        skip_new,
        getter_logic,
        skip_getter,
        return_type,
        copy
    )
)]
pub fn derive_getters_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let generics = &input.generics;
    let mut getters = Vec::new();
    let mut mut_getters = Vec::new();

    // Check if `skip_new` attribute is present.
    let mut skip_new = false;
    for attr in &input.attrs {
        if attr.path().is_ident(SKIP_NEW) {
            skip_new = true;
            break;
        }
    }

    // Generate getters based on struct fields and attributes.
    if let Data::Struct(data_struct) = &input.data {
        // Handle named fields.
        if let Fields::Named(fields_named) = &data_struct.fields {
            for f in fields_named.named.iter() {
                let field_name = f.ident.as_ref().unwrap();
                let field_ty = &f.ty;

                // Parse and process attributes for each field.
                let attrs = parse_field_attributes(&f.attrs);

                // Generate getters based on parsed attributes.
                if !attrs.skip_getter {
                    let getter = if let Some(logic_str) = attrs.custom_logic {
                        if let Some(custom_type) = &attrs.custom_return_type {
                            let logic: proc_macro2::TokenStream =
                                logic_str.parse().unwrap_or_else(|_| quote! {});
                            quote! {
                                pub fn #field_name(&self) -> #custom_type {
                                    #logic(self.#field_name)
                                }
                            }
                        } else {
                            let logic: proc_macro2::TokenStream =
                                logic_str.parse().unwrap_or_else(|_| quote! {});
                            quote! {
                                pub fn #field_name(&self) -> u32 {
                                    #logic(self.#field_name)
                                }
                            }
                        }
                    } else if attrs.copy {
                        if let Some(custom_type) = &attrs.custom_return_type {
                            quote! {
                                pub fn #field_name(&self) -> #custom_type {
                                    self.#field_name
                                }
                            }
                        } else {
                            quote! {
                                pub fn #field_name(&self) -> #field_ty {
                                    self.#field_name
                                }
                            }
                        }
                    } else if attrs.use_deref {
                        if let Some(custom_type) = &attrs.custom_return_type {
                            quote! {
                                pub fn #field_name(&self) -> #custom_type {
                                    &*self.#field_name
                                }
                            }
                        } else {
                            quote! {
                                pub fn #field_name(&self) -> &<#field_ty as std::ops::Deref>::Target {
                                    &*self.#field_name
                                }
                            }
                        }
                    } else if attrs.use_as_ref {
                        if let Some(custom_type) = &attrs.custom_return_type {
                            quote! {
                                pub fn #field_name(&self) -> #custom_type {
                                    self.#field_name.as_ref()
                                }
                            }
                        } else {
                            quote! {
                                pub fn #field_name(&self) -> &<#field_ty as std::convert::AsRef<#field_ty>>::Target {
                                    self.#field_name.as_ref()
                                }
                            }
                        }
                    } else {
                        #[allow(clippy::collapsible_else_if)]
                        if let Some(custom_type) = &attrs.custom_return_type {
                            quote! {
                                pub fn #field_name(&self) -> #custom_type {
                                    &self.#field_name
                                }
                            }
                        } else {
                            quote! {
                                pub fn #field_name(&self) -> &#field_ty {
                                    &self.#field_name
                                }
                            }
                        }
                    };

                    getters.push(getter);

                    // Generate mutable getters if needed.
                    if attrs.generate_mut {
                        let getter_mut_name =
                            Ident::new(&format!("{}_mut", field_name), field_name.span());
                        let getter_mut = quote! {
                            pub fn #getter_mut_name(&mut self) -> &mut #field_ty {
                                &mut self.#field_name
                            }
                        };
                        mut_getters.push(getter_mut);
                    }
                }
            }
        }
        // Handle unnamed fields (tuples).
        if let Fields::Unnamed(fields_unnamed) = &data_struct.fields {
            for (i, f) in fields_unnamed.unnamed.iter().enumerate() {
                let field_ty = &f.ty;
                let getter_name = Ident::new(&format!("get_{}", i), f.span());
                let index = syn::Index::from(i); // Using syn::Index::from
                let getter = quote! {
                    pub fn #getter_name(&self) -> &#field_ty {
                        &self.#index
                    }
                };
                getters.push(getter);
            }
        }
    }

    // Generate a `new` function if not skipped.
    let new_fn = if !skip_new {
        generate_new_fn(&input.data)
    } else {
        quote! {}
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Combine getters, mutable getters, and the `new` function into the impl block..
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #new_fn

            #(#getters)*
            #(#mut_getters)*
        }
    };

    // Convert to a TokenStream and return.
    TokenStream::from(expanded)
}

fn generate_new_fn(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let args = fields_named.named.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let field_ty = &f.ty;
                    quote! { #field_name: #field_ty }
                });
                let assignments = fields_named.named.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    quote! { #field_name: #field_name }
                });
                quote! {
                    pub fn new(#(#args),*) -> Self {
                        Self {
                            #(#assignments),*
                        }
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let args = fields_unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                    let field_ty = &f.ty;
                    let ident = Ident::new(&format!("field_{}", i), f.span());
                    quote! { #ident: #field_ty }
                });
                let assignments = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let ident = Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());
                    quote! { #ident }
                });
                quote! {
                    pub fn new(#(#args),*) -> Self {
                        Self(#(#assignments),*)
                    }
                }
            }
            Fields::Unit => quote! {},
        },
        Data::Enum(_) => quote! {},
        Data::Union(_) => quote! {},
    }
}

/// Represents parsed field attributes for getter generation.
#[derive(Default)]
struct FieldAttributes {
    use_deref: bool,
    use_as_ref: bool,
    generate_mut: bool,
    skip_getter: bool,
    custom_logic: Option<LitStr>,
    custom_return_type: Option<syn::Type>,
    copy: bool,
}

/// Parses attributes applied to struct fields and returns a `FieldAttributes` instance.
///
/// This function reads through the provided attributes and sets flags in `FieldAttributes`
/// based on the attributes found.
fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    attrs
        .iter()
        .fold(FieldAttributes::default(), |mut acc, attr| {
            match attr.meta {
                syn::Meta::NameValue(ref nv) if nv.path.is_ident(RETURN_TYPE) => {
                    if let syn::Expr::Lit(ref value) = nv.value {
                        match &value.lit {
                            syn::Lit::Str(ref lit) => {
                                acc.custom_return_type = lit.parse().ok();
                            }
                            _ => todo!(),
                        }
                    }
                }
                syn::Meta::Path(ref path) if path.is_ident(USE_DEREF) => acc.use_deref = true,
                syn::Meta::Path(ref path) if path.is_ident(COPY) => acc.copy = true,
                syn::Meta::Path(ref path) if path.is_ident(USE_AS_REF) => acc.use_as_ref = true,
                syn::Meta::Path(ref path) if path.is_ident(GET_MUT) => acc.generate_mut = true,
                syn::Meta::Path(ref path) if path.is_ident(SKIP_GETTER) => acc.skip_getter = true,
                syn::Meta::NameValue(ref nv) if nv.path.is_ident(GETTER_LOGIC) => {
                    if let syn::Expr::Lit(ref value) = nv.value {
                        match &value.lit {
                            syn::Lit::Str(lit) => acc.custom_logic = Some(lit.clone()),
                            _ => todo!(),
                        }
                    }
                }
                _ => (),
            }
            acc
        })
}
