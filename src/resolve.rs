use crate::{
    parse::{self, Id, Name, Arg, Ast, Args, ArgKind, Expr, ImportKind},
    token::Span,
    error::Error, 
    Document, 
    Imports,
    ComponentImport,
    InstanceImport,
    Instantiations,
    Instantiation,
    InstantiationArg,
    InstantiationArgs,
};
use anyhow::{
    anyhow, 
    bail, 
    Result,
    Context,
};
use wit_parser::{
    Resolve,
    PackageName,
    WorldId,
    PackageId,
    InterfaceId,
};
use std::fmt;

impl<'i> Ast<'i> {
    pub fn resolve(&self, resolve: &Resolve) -> Result<Document> {
        let imports = self.resolve_imports(resolve)?;
        let instantiations = self.resolve_instantiations(&imports)?;
        Ok(Document { 
            imports,
            instantiations,
        })
    }

    fn resolve_imports(&self, resolve: &Resolve) -> Result<Imports> {
        let mut imports = Imports::default();

        self.for_each_import(|name, kind| {
            match kind {
                ImportKind::Component(id) => {
                    let world_id = match id {
                        Some(_id) => todo!("resolve component import world id"),
                        None => None,
                    };

                    let component_import_id = imports
                        .components
                        .alloc(ComponentImport {
                            name: name.name.to_string(),
                            type_: world_id,
                        });
                    
                    assert!(imports
                        .component_names
                        .insert(name.name.to_string(), component_import_id)
                        .is_none());
                }
                ImportKind::Interface(id) => {
                    let (package_name, interface_id) = id.resolve_interface_id(resolve)?;
                
                    let instance_import_id = imports
                        .instances
                        .alloc(InstanceImport {
                            name: package_name.interface_id(id.element.name),
                            type_: interface_id,
                        });
                        
                    assert!(imports
                        .instance_names
                        .insert(name.name.to_string(), instance_import_id)
                        .is_none());
                }
            }
            Ok(())
        })?;

        Ok(imports)
    }

    fn resolve_instantiations(&self, imports: &Imports) -> Result<Instantiations> {
        let mut instantiations = Instantiations::default();

        self.for_each_instantiation(|name: &Name, expr| {
            let instantiation = expr.resolve(imports)?;
            // TODO: handle redefinition error
            assert!(instantiations
                .insert(name.name.to_string(), instantiation)
                .is_none());
            Ok(())
        })?;

        Ok(instantiations)
    }
}

impl<'i> Args<'i> {
    fn resolve(&self, imports: &Imports) -> Result<InstantiationArgs> {
        let mut instantiation_args = Default::default();

        for Arg { kind, .. } in self.0.iter() {
            match kind {
                ArgKind::Named { name, with } => {
                    todo!("resolve named instantiation argument")
                }
                ArgKind::Unnamed(expr) => {
                    todo!("resolve unnamed instantiation argument")
                }
            }
        }

        Ok(instantiation_args)
    }
}

impl<'i> Expr<'i> {
    fn resolve(&self, imports: &Imports) -> Result<Instantiation> {
        let Expr::Instantiate(name, args) = self else {
            bail!(self.error("found name", "expected instantiation"));
        };
        let component_id = imports
            .component_names
            .get(name.name)
            .ok_or(anyhow!(name.unresolved_err("component import", name.name)))?;

        let arguments = args.resolve(imports)?;

        Ok(Instantiation {
            component: *component_id,
            arguments,
        })
    }

    fn error(&self, kind: &'static str, msg: impl fmt::Display) -> Error {
        Error {
            msg: format!("bad expression {kind}; {msg}"),
            span: self.span(),
        }
    }
}

impl<'i> Name<'i> {
    fn unresolved_err(&self, kind: &'static str, msg: impl fmt::Display) -> Error {
        Error {
            msg: format!("unresolved {kind} name; {msg}"),
            span: self.span,
        }
    }
}

impl<'i> Id<'i> {
    fn resolve_interface_id(&self, resolve: &Resolve) -> Result<(PackageName, InterfaceId)> {
        let package_name = self.as_wit_package_name();

        let Some(package_id) = resolve
            .package_names
            .get(&package_name) else {
                anyhow::bail!(self.unresolved_err(
                    "interface", 
                    format!("could not find package {package_name}")))
            };

        let interface_id = resolve
            .packages[*package_id]
            .interfaces
            .get(self.element.name)
            .copied()
            .ok_or(anyhow!(self.unresolved_err("interface", self.element.name)))?;

        Ok((package_name, interface_id))
    }

    fn unresolved_err(&self, kind: &'static str, msg: impl fmt::Display) -> Error {
        assert!(self.version.is_none());
        Error {
            msg: format!("unresolved {kind} name; {msg}"),
            span: Span {
                start: self.namespace.span.start,
                end: self.element.span.end,
            },
        }
    }
}