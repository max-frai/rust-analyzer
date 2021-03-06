use itertools::Itertools;
use ra_syntax::{
    TextRange, SyntaxNode,
    ast::{self, AstNode, NameOwner, ModuleItemOwner},
};
use ra_db::{Cancelable, SyntaxDatabase};

use crate::{db::RootDatabase, FileId};

#[derive(Debug)]
pub struct Runnable {
    pub range: TextRange,
    pub kind: RunnableKind,
}

#[derive(Debug)]
pub enum RunnableKind {
    Test { name: String },
    TestMod { path: String },
    Bench { name: String },
    Bin,
}

pub(crate) fn runnables(db: &RootDatabase, file_id: FileId) -> Cancelable<Vec<Runnable>> {
    let source_file = db.source_file(file_id);
    let res = source_file
        .syntax()
        .descendants()
        .filter_map(|i| runnable(db, file_id, i))
        .collect();
    Ok(res)
}

fn runnable(db: &RootDatabase, file_id: FileId, item: &SyntaxNode) -> Option<Runnable> {
    if let Some(fn_def) = ast::FnDef::cast(item) {
        runnable_fn(fn_def)
    } else if let Some(m) = ast::Module::cast(item) {
        runnable_mod(db, file_id, m)
    } else {
        None
    }
}

fn runnable_fn(fn_def: &ast::FnDef) -> Option<Runnable> {
    let name = fn_def.name()?.text();
    let kind = if name == "main" {
        RunnableKind::Bin
    } else if fn_def.has_atom_attr("test") {
        RunnableKind::Test {
            name: name.to_string(),
        }
    } else if fn_def.has_atom_attr("bench") {
        RunnableKind::Bench {
            name: name.to_string(),
        }
    } else {
        return None;
    };
    Some(Runnable {
        range: fn_def.syntax().range(),
        kind,
    })
}

fn runnable_mod(db: &RootDatabase, file_id: FileId, module: &ast::Module) -> Option<Runnable> {
    let has_test_function = module
        .item_list()?
        .items()
        .filter_map(|it| match it.kind() {
            ast::ModuleItemKind::FnDef(it) => Some(it),
            _ => None,
        })
        .any(|f| f.has_atom_attr("test"));
    if !has_test_function {
        return None;
    }
    let range = module.syntax().range();
    let module =
        hir::source_binder::module_from_child_node(db, file_id, module.syntax()).ok()??;

    // FIXME: thread cancellation instead of `.ok`ing
    let path = module
        .path_to_root(db)
        .ok()?
        .into_iter()
        .rev()
        .filter_map(|it| it.name(db).ok())
        .filter_map(|it| it)
        .join("::");
    Some(Runnable {
        range,
        kind: RunnableKind::TestMod { path },
    })
}
