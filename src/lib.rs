use proc_macro::Span;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[proc_macro]
pub fn notifbot_enum(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let (struct_name, variants) = parse_macro_input(input_str);
    let enum_variants: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let variant_lower = variant.to_lowercase();
            let ident = format_ident!("{}", variant);
            let value = idx as u8;
            quote! {
                #[serde(alias = #variant, alias = #variant_lower)]
                #ident = #value
            }
        })
        .collect();

    let from_impl_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let ident = format_ident!("{}", variant);
            let value = idx as i8;
            quote! {
                #value => #struct_name::#ident,
            }
        })
        .collect();

    let into_impl_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let ident = format_ident!("{}", variant);
            let value = idx as i8;
            quote! {
                #struct_name::#ident => #value,
            }
        })
        .collect();

    let into_str_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let ident = format_ident!("{}", variant);
            let variant_lower = variant.to_lowercase();
            quote! {
                #struct_name::#ident => #variant_lower,
            }
        })
        .collect();

    let fmt_arms: Vec<_> = variants.iter().map(|variant| {
        let ident = format_ident!("{}", variant);
        quote! {
            #struct_name::#ident => f.write_fmt(format_args!("{}", #variant)),
        }
    }).collect();

    let output = quote! {
        #[repr(u8)]
        #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        pub enum #struct_name {
            #(#enum_variants,)*
        }

        impl From<#struct_name> for i8 {
            fn from(v: #struct_name) -> i8 {
                match v {
                    #(#into_impl_arms)*
                }
            }
        }

        impl From<&#struct_name> for i8 {
            fn from(v: &#struct_name) -> i8 {
                match v {
                    #(#into_impl_arms)*
                }
            }
        }

        impl From<i8> for #struct_name {
            fn from(v: i8) -> #struct_name {
                match v {
                    #(#from_impl_arms)*
                    _ => panic!()
                }
            }
        }

        impl From<#struct_name> for &'static str {
            fn from(v: #struct_name) -> &'static str {
                match v {
                    #(#into_str_arms)*
                }
            }
        }

        impl From<&#struct_name> for &'static str {
            fn from(v: &#struct_name) -> &'static str {
                match v {
                    #(#into_str_arms)*
                }
            }
        }

        impl #struct_name {
            pub fn to_str(&self) -> &'static str {
                self.into()
            }
        }

        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                #(#fmt_arms)*           
                }
      
            }
        }
    };

    TokenStream::from(output)
}

fn parse_macro_input(input: String) -> (Ident, Vec<String>) {
    let parts: Vec<_> = input.split_whitespace().collect();

    let struct_name = Ident::new(parts[0].trim_end_matches("("), Span::call_site().into());

    let binding = parts[1..].join(" ");
    let variants_raw = &binding.trim_matches(['{', '}', ')']);
    let variants: Vec<_> = variants_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    (struct_name, variants)
}
