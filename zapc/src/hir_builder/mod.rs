use chumsky::span;
use lasso::Rodeo;
use scope::Resolved;

use crate::{
	ast::Ast,
	hir::{decl::HirRemote, scope::HirScope, ty::HirTy, Hir},
	meta::Report,
};

mod decl;
mod range;
mod scope;
mod ty;

pub struct HirBuilder<'a> {
	reports: Vec<Report>,
	rodeo: &'a mut Rodeo,

	init_scope: HirScope,
	ty_decls: Vec<Resolved<HirTy>>,
	remote_decls: Vec<Resolved<HirRemote>>,
}

impl<'a> HirBuilder<'a> {
	pub fn new(rodeo: &'a mut Rodeo) -> Self {
		Self {
			rodeo,
			reports: Vec::new(),
			init_scope: Default::default(),
			ty_decls: Vec::new(),
			remote_decls: Vec::new(),
		}
	}

	pub fn init_ast(mut self, ast: Ast) -> Result<Hir, Vec<Report>> {
		self.decls(&Self::INIT_SCOPEID, ast.into_decls());

		let HirBuilder {
			mut reports,
			ty_decls,
			remote_decls,
			..
		} = self;

		let mut resolved_ty_decls = Vec::new();

		for ty in ty_decls {
			match ty {
				Resolved::Resolved(_, ty) => {
					resolved_ty_decls.push(ty);
				}
				Resolved::Unresolved(spans) => {
					let span = spans.first().unwrap().merge(*spans.last().unwrap());

					reports.push(Report::UnknownType { type_span: span });
				}
			}
		}

		let mut resolved_remote_decls = Vec::new();

		for remote in remote_decls {
			match remote {
				Resolved::Resolved(_, remote) => {
					resolved_remote_decls.push(remote);
				}
				Resolved::Unresolved(spans) => {
					let span = spans.first().unwrap().merge(*spans.last().unwrap());

					reports.push(Report::UnknownRemote { remote_span: span });
				}
			}
		}

		if reports.is_empty() {
			Ok(Hir::new(self.init_scope, resolved_ty_decls, resolved_remote_decls))
		} else {
			Err(reports)
		}
	}

	fn report(&mut self, report: Report) {
		self.reports.push(report);
	}
}
