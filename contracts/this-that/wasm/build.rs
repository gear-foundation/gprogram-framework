use convert_case::{Case, Casing};
use gmeta::Metadata;
use handlebars::{handlebars_helper, Handlebars};
use parity_scale_codec::Decode;
use scale_info::{PortableRegistry, PortableType, TypeDef};
use serde_json;
use std::fs;

#[derive(serde::Serialize)]
struct TypeData<'a> {
    types: &'a Vec<&'a PortableType>,
    commands: &'a PortableType,
    #[serde(rename = "commandResponses")]
    command_responses: &'a PortableType,
    registry: &'a Vec<String>,
}

handlebars_helper!(deref: |v: String| { v });

fn main() {
    gwasm_builder::build_with_metadata::<this_that_app::ProgramMetadata>();

    let meta_repr = this_that_app::ProgramMetadata::repr();
    let mut encoded_registry = meta_repr.registry.as_slice();
    let portable_registry: PortableRegistry =
        PortableRegistry::decode(&mut encoded_registry).unwrap();

    let all_type_names = &portable_registry
        .types
        .iter()
        .map(|ty| get_type_name_from_registry(&portable_registry, ty.id))
        .collect::<Vec<_>>();

    let all_named_types = &portable_registry
        .types
        .iter()
        .filter(|ty| !ty.ty.path.namespace().is_empty())
        .collect::<Vec<_>>();

    let command_type = &portable_registry
        .types
        .iter()
        .find(|ty| matches!(ty.ty.path.ident(), Some(ident) if ident == "Commands"))
        .unwrap();

    let command_reponses_type = &portable_registry
        .types
        .iter()
        .find(|ty| matches!(ty.ty.path.ident(), Some(ident) if ident == "CommandResponses"))
        .unwrap();

    let type_data = TypeData {
        types: all_named_types,
        commands: command_type,
        command_responses: command_reponses_type,
        registry: all_type_names,
    };

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("idl", "/home/dennis/sources/gear-tech/gprogram-framework/contracts/this-that/wasm/hbs/idl.hbs")
        .unwrap();
    handlebars
        .register_template_file("composite", "/home/dennis/sources/gear-tech/gprogram-framework/contracts/this-that/wasm/hbs/composite.hbs")
        .unwrap();
    handlebars
        .register_template_file("variant", "/home/dennis/sources/gear-tech/gprogram-framework/contracts/this-that/wasm/hbs/variant.hbs")
        .unwrap();
    handlebars.register_helper("deref", Box::new(deref));

    let idl_file = fs::File::create("/home/dennis/this-that.idl").unwrap();
    handlebars
        .render_to_write("idl", &type_data, idl_file)
        .unwrap();
}

fn get_type_name_from_registry(type_registry: &PortableRegistry, type_id: u32) -> String {
    let ty = type_registry.resolve(type_id).unwrap();

    if !ty.path.is_empty() {
        let type_name = ty.path.segments.join("");
        match type_name.as_str() {
            "Result" => {
                let result_params = ty
                    .type_params
                    .iter()
                    .map(|type_param| {
                        get_type_name_from_registry(type_registry, type_param.ty.unwrap().id)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                return format!("({})", result_params);
            }
            "Option" => {
                let option_param =
                    get_type_name_from_registry(type_registry, ty.type_params[0].ty.unwrap().id);
                return format!("opt {}", option_param);
            }
            _ => {
                return type_name.to_case(Case::Pascal);
            }
        }
    }

    match &ty.type_def {
        TypeDef::Primitive(primitive_def) => {
            let type_name = serde_json::to_string(&primitive_def)
                .unwrap()
                .trim_matches('"')
                .into();
            if type_name == "str" {
                "text".into()
            } else {
                type_name
            }
        }
        TypeDef::Tuple(tuple_def) => {
            let fields = tuple_def
                .fields
                .iter()
                .map(|field| get_type_name_from_registry(type_registry, field.id))
                .collect::<Vec<_>>()
                .join("; ");
            format!("record {{ {} }}", fields)
        }
        _ => "Unknown".into(),
    }
}
