//! File-based module resolution. Each dependency is elaborated in its own scope.
//! Linking typed declarations does not change Horn, effects or default selection.
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::elab::{Error, elaborate_formal_with_dependencies};
use crate::surface::{FormalDeclaration, FormalModule};
use crate::typed::{Declaration, DeclarationOrigin, Module, Type};

const MAX_SOURCE_BYTES: usize = 64 * 1024;
const MAX_MODULES: usize = 64;

/// Resolve paths relative to the importing file. All non-experiment symbols,
/// including transitive dependencies, are public in this first module system.
/// Unique files are bounded together, not by an independent limit per import.
pub fn load_formal_module(path: impl AsRef<Path>) -> Result<Module, Error> {
    Resolver::default().load(path.as_ref())
}

#[derive(Default)]
struct Resolver {
    loaded: BTreeMap<PathBuf, Module>,
    active: Vec<PathBuf>,
    bytes: usize,
}

impl Resolver {
    fn load(&mut self, requested: &Path) -> Result<Module, Error> {
        let path = requested.canonicalize().map_err(|e| {
            Error::new(format!(
                "cannot resolve module `{}`: {e}",
                requested.display()
            ))
        })?;
        if let Some(start) = self.active.iter().position(|p| p == &path) {
            let cycle = self.active[start..]
                .iter()
                .chain(std::iter::once(&path))
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(Error::new(format!("cyclic import: {cycle}")));
        }
        if let Some(module) = self.loaded.get(&path) {
            return Ok(module.clone());
        }
        if self.active.len() + self.loaded.len() >= MAX_MODULES {
            return Err(Error::new("module graph exceeds 64 unique files"));
        }
        let file = std::fs::File::open(&path).map_err(|e| Error::new(e.to_string()))?;
        if !file
            .metadata()
            .map_err(|e| Error::new(e.to_string()))?
            .is_file()
        {
            return Err(Error::new(format!(
                "module `{}` is not a regular file",
                path.display()
            )));
        }
        let remaining = MAX_SOURCE_BYTES - self.bytes;
        let mut source = String::new();
        file.take(remaining as u64 + 1)
            .read_to_string(&mut source)
            .map_err(|e| Error::new(format!("cannot read module `{}`: {e}", path.display())))?;
        if source.len() > remaining {
            return Err(Error::new("module graph exceeds 64 KiB of source"));
        }
        self.bytes += source.len();
        let surface = crate::parse_formal(&source)
            .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
        self.active.push(path.clone());
        let result = self.resolve(surface, &path);
        self.active.pop();
        let module = result?;
        self.loaded.insert(path, module.clone());
        Ok(module)
    }

    fn resolve(&mut self, surface: FormalModule, path: &Path) -> Result<Module, Error> {
        let mut imported = Vec::new();
        let mut origins = BTreeMap::<String, DeclarationOrigin>::new();
        for declaration in &surface.declarations {
            let FormalDeclaration::Import(relative) = declaration else {
                continue;
            };
            if relative.is_empty() || Path::new(relative).is_absolute() {
                return Err(Error::new("import requires a nonempty relative path"));
            }
            let dependency = self.load(&path.parent().unwrap().join(relative))?;
            for declaration in &dependency.declarations {
                if matches!(declaration, Declaration::Experiment(_)) {
                    continue;
                }
                let name = declaration.name();
                let origin = dependency.origin(name).unwrap();
                if let Some(previous) = origins.get(name) {
                    if previous == origin {
                        continue;
                    } // diamond/repeated import
                    return Err(duplicate(name, previous, origin));
                }
                origins.insert(name.to_string(), origin.clone());
                imported.push(declaration.clone());
            }
        }
        let origin = DeclarationOrigin {
            module: surface.name.clone(),
            file: Some(path.to_path_buf()),
        };
        let mut declarations = Vec::new();
        for declaration in surface.declarations {
            if matches!(declaration, FormalDeclaration::Import(_)) {
                continue;
            }
            let name = declaration.name().unwrap().to_string();
            if let Some(previous) = origins.get(&name) {
                return Err(duplicate(&name, previous, &origin));
            }
            origins.insert(name, origin.clone());
            declarations.push(declaration);
        }
        validate_types(&declarations, &imported)?;
        let mut module = elaborate_formal_with_dependencies(
            FormalModule {
                name: surface.name,
                declarations,
            },
            &imported,
        )
        .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
        module.origins = origins;
        Ok(module)
    }
}

fn duplicate(name: &str, left: &DeclarationOrigin, right: &DeclarationOrigin) -> Error {
    Error::new(format!(
        "duplicate symbol `{name}` from {} ({:?}) and {} ({:?})",
        left.module, left.file, right.module, right.file
    ))
}

/// Entity references must resolve to declarations, rather than implicitly
/// becoming entities through a type mention. Forward local record types remain
/// valid; sibling or consumer symbols are never part of a module's scope.
fn validate_types(local: &[FormalDeclaration], imported: &[Declaration]) -> Result<(), Error> {
    let mut entities: BTreeSet<&str> = imported
        .iter()
        .filter_map(|d| match d {
            Declaration::Entity(e) => Some(e.name.as_str()),
            _ => None,
        })
        .collect();
    entities.extend(local.iter().filter_map(|d| match d {
        FormalDeclaration::Entity(e) => Some(e.name.as_str()),
        _ => None,
    }));
    fn check(ty: &Type, entities: &BTreeSet<&str>) -> Result<(), Error> {
        match ty {
            Type::Entity(name) if !entities.contains(name.as_str()) => {
                Err(Error::new(format!("unknown entity `{name}`")))
            }
            Type::Set(element) | Type::Optional(element) => check(element, entities),
            _ => Ok(()),
        }
    }
    for declaration in local {
        match declaration {
            FormalDeclaration::Entity(e) => {
                for parent in &e.supertypes {
                    check(&Type::Entity(parent.clone()), &entities)?;
                }
                for field in &e.fields {
                    check(&field.ty, &entities)?;
                }
            }
            FormalDeclaration::Const(c) => check(&c.ty, &entities)?,
            FormalDeclaration::Verb(v) => {
                for arg in &v.args {
                    check(&arg.ty, &entities)?;
                }
            }
            FormalDeclaration::Program(p) => {
                if let crate::surface::SurfaceProgramBody::Definition { parameters, .. } = &p.body {
                    for arg in parameters {
                        check(&arg.ty, &entities)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
