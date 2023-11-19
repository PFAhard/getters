extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Ident};

// Custom attributes, for illustrative purposes
const USE_DEREF: &str = "use_deref";
const USE_AS_REF: &str = "use_as_ref";
const GET_MUT: &str = "get_mut";
const SKIP_NEW: &str = "skip_new";
const GETTER_LOGIC: &str = "getter_logic";

#[proc_macro_derive(
    Getters,
    attributes(use_deref, use_as_ref, get_mut, skip_new, getter_logic)
)]
pub fn derive_getters_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut getters = Vec::new();
    let mut mut_getters = Vec::new();

    let mut skip_new = false;
    for attr in &input.attrs {
        if attr.path().is_ident(SKIP_NEW) {
            skip_new = true;
            break;
        }
    }

    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            for f in fields_named.named.iter() {
                let field_name = f.ident.as_ref().unwrap();
                let field_ty = &f.ty;

                let mut use_deref = false;
                let mut use_as_ref = false;
                let mut generate_mut = false;
                let mut custom_logic = None;

                // Check attributes to set flags for getter generation
                for attr in &f.attrs {
                    if attr.path().is_ident(USE_DEREF) {
                        use_deref = true;
                    } else if attr.path().is_ident(USE_AS_REF) {
                        use_as_ref = true;
                    } else if attr.path().is_ident(GET_MUT) {
                        generate_mut = true;
                    } else if attr.path().is_ident(GETTER_LOGIC) {
                        if let syn::Meta::NameValue(meta_name_value) = &attr.meta {
                            if let syn::Expr::Lit(lit_str) = &meta_name_value.value {
                                match &lit_str.lit {
                                    syn::Lit::Str(lit) => {
                                        custom_logic = Some(lit);
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                    }
                }

                // Generate the appropriate immutable getter based on the attributes
                let getter = if let Some(logic_str) = custom_logic {
                    let logic: proc_macro2::TokenStream = logic_str.parse().unwrap_or_else(|_| quote! {});
                    quote! {
                        pub fn #field_name(&self) -> u32 {
                            #logic(self.#field_name)
                        }
                    }
                } else if use_deref {
                    quote! {
                        pub fn #field_name(&self) -> &<#field_ty as std::ops::Deref>::Target {
                            &*self.#field_name
                        }
                    }
                } else if use_as_ref {
                    quote! {
                        pub fn #field_name(&self) -> &<#field_ty as std::convert::AsRef<#field_ty>>::Target {
                            self.#field_name.as_ref()
                        }
                    }
                } else {
                    quote! {
                        pub fn #field_name(&self) -> &#field_ty {
                            &self.#field_name
                        }
                    }
                };

                getters.push(getter);

                // Generate mutable getter if the get_mut attribute is present
                if generate_mut {
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

    // Generate the new function with fields and types.
    let new_fn = if !skip_new {
        generate_new_fn(&input.data)
    } else {
        quote! {}
    };

    // Combine the getters and mutable getters and the new function into the final impl block.
    let expanded = quote! {
        impl #name {
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
