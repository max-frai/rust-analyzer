use ra_db::{Cancelable, FilePosition};

use crate::{NavigationTarget, db::RootDatabase};

/// This returns `Vec` because a module may be included from several places. We
/// don't handle this case yet though, so the Vec has length at most one.
pub(crate) fn parent_module(
    db: &RootDatabase,
    position: FilePosition,
) -> Cancelable<Vec<NavigationTarget>> {
    let module = match hir::source_binder::module_from_position(db, position)? {
        None => return Ok(Vec::new()),
        Some(it) => it,
    };
    let nav = NavigationTarget::from_module(db, module)?;
    Ok(vec![nav])
}

#[cfg(test)]
mod tests {
    use crate::mock_analysis::analysis_and_position;

    #[test]
    fn test_resolve_parent_module() {
        let (analysis, pos) = analysis_and_position(
            "
            //- /lib.rs
            mod foo;
            //- /foo.rs
            <|>// empty
            ",
        );
        let nav = analysis.parent_module(pos).unwrap().pop().unwrap();
        nav.assert_match("foo SOURCE_FILE FileId(2) [0; 10)");
    }

    #[test]
    fn test_resolve_parent_module_for_inline() {
        let (analysis, pos) = analysis_and_position(
            "
            //- /lib.rs
            mod foo {
                mod bar {
                    mod baz { <|> }
                }
            }
            ",
        );
        let nav = analysis.parent_module(pos).unwrap().pop().unwrap();
        nav.assert_match("baz MODULE FileId(1) [32; 44)");
    }
}
