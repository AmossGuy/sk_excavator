use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{*, spanned::Spanned};

pub fn macro_main(input: DeriveInput) -> TokenStream {
	let methods = match input.data {
		Data::Struct(struct_data) => struct_methods(struct_data),
		Data::Enum(enum_data) => enum_methods(enum_data),
		Data::Union(union_data) => {
			let span = union_data.union_token.span;
			return quote_spanned! {span=>
				compile_error!("EditableData derive does not support unions");
			};
		},
	};
	
	let Methods { field_count, field_name, field_ref } = methods;
	let input_ident = input.ident;
	let input_ident_string = input_ident.to_string();
	
	quote! {
		#[automatically_derived]
		impl crate::formats::common::EditableData for #input_ident {
			fn struct_name(&self) -> &str {
				#input_ident_string
			}
			
			fn field_count(&self) -> usize { #field_count }
			fn field_name(&self, index: usize) -> &str { #field_name }
			fn field_ref(&self, index: usize) -> crate::formats::common::FieldRef<'_> { #field_ref }
			
			/*
			fn variant_count(&self) -> usize {
				todo!()
			}
			
			fn variant_name(&self, index: usize) -> &str {
				todo!()
			}
			
			fn variant_current(&self) -> usize {
				todo!()
			}
			*/
		}
	}
}

struct Methods {
	field_count: TokenStream,
	field_name: TokenStream,
	field_ref: TokenStream,
}

fn struct_methods(struct_data: DataStruct) -> Methods {
	let fields = struct_data.fields.iter().enumerate().map(|(i, field)| {
		let values = AttributeValues::parse(field.attrs.iter());
		(i, field, values)
	}).collect::<Vec<_>>();
	
	let field_count = fields.len();
	
	Methods {
		field_count: quote! { #field_count },
		field_name: quote! { todo!() },
		field_ref: quote! { todo!() },
	}
}

fn enum_methods(enum_data: DataEnum) -> Methods {
	todo!("enum_methods")
}

struct AttributeValues {
	skip: bool,
	parse_errors: Vec<syn::Error>,
}

impl AttributeValues {
	fn parse<'a>(attrs: impl Iterator<Item = &'a Attribute>) -> Self {
		let mut this = Self {
			skip: false,
			parse_errors: Vec::new(),
		};
		
		for attr in attrs {
			if attr.path().is_ident("edit") {
				let result = attr.parse_nested_meta(|meta| {
					if meta.path.is_ident("skip") {
						this.skip = true;
						Ok(())
					} else {
						Err(meta.error("unrecognized `edit` attribute property"))
					}
				});
				
				if let Err(e) = result {
					this.parse_errors.push(e);
				}
			}
		}
		
		this
	}
}
