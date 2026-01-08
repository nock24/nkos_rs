use syn::{
    Ident, LitInt, Token, Type, Visibility, braced, bracketed,
    parse::{Parse, ParseStream},
};

pub struct SectorLayout {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: Ident,
    pub ty: FieldType,
}

pub enum FieldType {
    Fixed(Type),
    FixedArr { elem_ty: Type, len: usize },
    DynArr { elem_ty: Type, len_ident: Ident },
}

impl Parse for SectorLayout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;

        let contents;
        braced!(contents in input);

        let mut fields = Vec::new();
        while !contents.is_empty() {
            let field_name: Ident = contents.parse()?;
            contents.parse::<Token![:]>()?;

            // Array types: [T; N] or [T; len_ident]
            if contents.peek(syn::token::Bracket) {
                let inner;
                bracketed!(inner in contents);

                let elem_ty: Type = inner.parse()?;
                inner.parse::<Token![;]>()?;

                if inner.peek(Ident) {
                    let len_ident: Ident = inner.parse()?;
                    fields.push(Field {
                        name: field_name,
                        ty: FieldType::DynArr { elem_ty, len_ident },
                    });
                } else {
                    let lit: LitInt = inner.parse()?;
                    let len = lit.base10_parse::<usize>()?;
                    fields.push(Field {
                        name: field_name,
                        ty: FieldType::FixedArr { elem_ty, len },
                    });
                }
            } else {
                let ty: Type = contents.parse()?;
                fields.push(Field {
                    name: field_name,
                    ty: FieldType::Fixed(ty),
                });
            }

            // optional comma
            let _ = contents.parse::<Token![,]>();
        }

        Ok(SectorLayout { vis, name, fields })
    }
}
