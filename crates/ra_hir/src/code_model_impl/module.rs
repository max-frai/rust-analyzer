use ra_db::{Cancelable, SourceRootId, FileId};
use ra_syntax::{ast, SyntaxNode, AstNode, TreeArc};

use crate::{
    Module, ModuleSource, Problem,
    Crate, DefId, DefLoc, DefKind, Name, Path, PathKind, PerNs, Def,
    module_tree::ModuleId,
    nameres::ModuleScope,
    db::HirDatabase,
};

impl Module {
    pub(crate) fn new(def_id: DefId) -> Self {
        crate::code_model_api::Module { def_id }
    }

    pub(crate) fn from_module_id(
        db: &impl HirDatabase,
        source_root_id: SourceRootId,
        module_id: ModuleId,
    ) -> Cancelable<Self> {
        let module_tree = db.module_tree(source_root_id)?;
        let def_loc = DefLoc {
            kind: DefKind::Module,
            source_root_id,
            module_id,
            source_item_id: module_id.source(&module_tree),
        };
        let def_id = def_loc.id(db);
        let module = Module::new(def_id);
        Ok(module)
    }

    pub(crate) fn name_impl(&self, db: &impl HirDatabase) -> Cancelable<Option<Name>> {
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        let link = ctry!(loc.module_id.parent_link(&module_tree));
        Ok(Some(link.name(&module_tree).clone()))
    }

    pub fn definition_source_impl(
        &self,
        db: &impl HirDatabase,
    ) -> Cancelable<(FileId, ModuleSource)> {
        let loc = self.def_id.loc(db);
        let file_id = loc.source_item_id.file_id.as_original_file();
        let syntax_node = db.file_item(loc.source_item_id);
        let module_source = if let Some(source_file) = ast::SourceFile::cast(&syntax_node) {
            ModuleSource::SourceFile(source_file.to_owned())
        } else {
            let module = ast::Module::cast(&syntax_node).unwrap();
            ModuleSource::Module(module.to_owned())
        };
        Ok((file_id, module_source))
    }

    pub fn declaration_source_impl(
        &self,
        db: &impl HirDatabase,
    ) -> Cancelable<Option<(FileId, TreeArc<ast::Module>)>> {
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        let link = ctry!(loc.module_id.parent_link(&module_tree));
        let file_id = link
            .owner(&module_tree)
            .source(&module_tree)
            .file_id
            .as_original_file();
        let src = link.source(&module_tree, db);
        Ok(Some((file_id, src)))
    }

    pub(crate) fn krate_impl(&self, db: &impl HirDatabase) -> Cancelable<Option<Crate>> {
        let root = self.crate_root(db)?;
        let loc = root.def_id.loc(db);
        let file_id = loc.source_item_id.file_id.as_original_file();

        let crate_graph = db.crate_graph();
        let crate_id = ctry!(crate_graph.crate_id_for_crate_root(file_id));
        Ok(Some(Crate::new(crate_id)))
    }

    pub(crate) fn crate_root_impl(&self, db: &impl HirDatabase) -> Cancelable<Module> {
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        let module_id = loc.module_id.crate_root(&module_tree);
        Module::from_module_id(db, loc.source_root_id, module_id)
    }

    /// Finds a child module with the specified name.
    pub fn child_impl(&self, db: &impl HirDatabase, name: &Name) -> Cancelable<Option<Module>> {
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        let child_id = ctry!(loc.module_id.child(&module_tree, name));
        Module::from_module_id(db, loc.source_root_id, child_id).map(Some)
    }

    /// Iterates over all child modules.
    pub fn children_impl(&self, db: &impl HirDatabase) -> Cancelable<impl Iterator<Item = Module>> {
        // FIXME this should be implementable without collecting into a vec, but
        // it's kind of hard since the iterator needs to keep a reference to the
        // module tree.
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        let children = loc
            .module_id
            .children(&module_tree)
            .map(|(_, module_id)| Module::from_module_id(db, loc.source_root_id, module_id))
            .collect::<Cancelable<Vec<_>>>()?;
        Ok(children.into_iter())
    }

    pub fn parent_impl(&self, db: &impl HirDatabase) -> Cancelable<Option<Module>> {
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        let parent_id = ctry!(loc.module_id.parent(&module_tree));
        Module::from_module_id(db, loc.source_root_id, parent_id).map(Some)
    }

    /// Returns a `ModuleScope`: a set of items, visible in this module.
    pub fn scope_impl(&self, db: &impl HirDatabase) -> Cancelable<ModuleScope> {
        let loc = self.def_id.loc(db);
        let item_map = db.item_map(loc.source_root_id)?;
        let res = item_map.per_module[&loc.module_id].clone();
        Ok(res)
    }

    pub fn resolve_path_impl(
        &self,
        db: &impl HirDatabase,
        path: &Path,
    ) -> Cancelable<PerNs<DefId>> {
        let mut curr_per_ns = PerNs::types(
            match path.kind {
                PathKind::Crate => self.crate_root(db)?,
                PathKind::Self_ | PathKind::Plain => self.clone(),
                PathKind::Super => {
                    if let Some(p) = self.parent(db)? {
                        p
                    } else {
                        return Ok(PerNs::none());
                    }
                }
            }
            .def_id,
        );

        let segments = &path.segments;
        for (idx, name) in segments.iter().enumerate() {
            let curr = if let Some(r) = curr_per_ns.as_ref().take_types() {
                r
            } else {
                return Ok(PerNs::none());
            };
            let module = match curr.resolve(db)? {
                Def::Module(it) => it,
                Def::Enum(e) => {
                    if segments.len() == idx + 1 {
                        // enum variant
                        let matching_variant =
                            e.variants(db)?.into_iter().find(|(n, _variant)| n == name);

                        if let Some((_n, variant)) = matching_variant {
                            return Ok(PerNs::both(variant.def_id(), e.def_id()));
                        } else {
                            return Ok(PerNs::none());
                        }
                    } else if segments.len() == idx {
                        // enum
                        return Ok(PerNs::types(e.def_id()));
                    } else {
                        // malformed enum?
                        return Ok(PerNs::none());
                    }
                }
                _ => return Ok(PerNs::none()),
            };
            let scope = module.scope(db)?;
            curr_per_ns = if let Some(r) = scope.get(&name) {
                r.def_id
            } else {
                return Ok(PerNs::none());
            };
        }
        Ok(curr_per_ns)
    }

    pub fn problems_impl(
        &self,
        db: &impl HirDatabase,
    ) -> Cancelable<Vec<(TreeArc<SyntaxNode>, Problem)>> {
        let loc = self.def_id.loc(db);
        let module_tree = db.module_tree(loc.source_root_id)?;
        Ok(loc.module_id.problems(&module_tree, db))
    }
}
