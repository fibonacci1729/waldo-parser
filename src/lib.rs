// use wit_parser::{Resolve, InterfaceId};

use anyhow::Result;
use id_arena::{Id, Arena};
use indexmap::IndexMap;
use std::path::Path;
use wit_parser::{Resolve, WorldId, InterfaceId};

mod error;
mod parse;
mod resolve;
mod token;

pub type ComponentImportId = Id<ComponentImport>;
pub type InstanceImportId = Id<InstanceImport>;

/// A component import.
pub struct ComponentImport {
    pub name: String,
    pub type_: Option<WorldId>,
}

/// An instance import.
pub struct InstanceImport {
    pub name: String,
    pub type_: InterfaceId,
}

/// An instantiation.
pub struct Instantiation {
    /// The component to be instantiation.
    pub component: ComponentImportId,
    /// Arguments to instantiate the component.
    pub arguments: InstantiationArgs,
}

/// An instantiation argument.
pub enum InstantiationArg {
    InstanceExport(InstanceImportId, String),
    Instance(InstanceImportId),
}

/// Ordered set of instantiations as they appear in a document.
pub type Instantiations = IndexMap<String, Instantiation>;

/// Ordered set of instantiation arguments.
pub type InstantiationArgs = IndexMap<String, InstantiationArg>;

/// The set of imports for a document.
#[derive(Default)]
pub struct Imports {
    pub component_names: IndexMap<String, ComponentImportId>,
    pub instance_names: IndexMap<String, InstanceImportId>,
    pub components: Arena<ComponentImport>,
    pub instances: Arena<InstanceImport>,
}

/// A resolve composition document.
#[derive(Default)]
pub struct Document {
    pub imports: Imports,
    pub instantiations: Instantiations,
}

impl Document {
    pub fn parse(resolve: &Resolve, path: impl AsRef<Path>, contents: &str) -> Result<Self> {
        let mut lex = token::Tokenizer::new(contents, 0)?;

        let path = path.as_ref();

        let ast = match parse::Ast::parse(&mut lex) {
            Ok(ast) => ast,
            Err(mut err) => {
                let file = path.display().to_string();
                error::rewrite(&mut err, &file, contents);
                return Err(err);
            }
        };

        match ast.resolve(resolve) {
            Ok(document) => Ok(document),
            Err(mut err) => {
                let file = path.display().to_string();
                error::rewrite(&mut err, &file, contents);
                return Err(err);
            }
        }
    }
}