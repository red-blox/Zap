use crate::{
	config::{Enum, Ty},
	irgen::Stmt,
};

pub mod client;
pub mod server;

pub trait Output {
	fn push(&mut self, s: &str);
	fn indent(&mut self);
	fn dedent(&mut self);
	fn push_indent(&mut self);

	fn push_line(&mut self, s: &str) {
		self.push_indent();
		self.push(s);
		self.push("\n");
	}

	fn push_stmt(&mut self, stmt: &Stmt) {
		if matches!(stmt, Stmt::ElseIf(..) | Stmt::Else | Stmt::End) {
			self.dedent();
		}

		match &stmt {
			Stmt::Local(name, expr) => {
				if let Some(expr) = expr {
					self.push_line(&format!("local {name} = {expr}"));
				} else {
					self.push_line(&format!("local {name}"));
				}
			}
			Stmt::LocalTuple(var, expr) => {
				let items = var.join(", ");

				if let Some(expr) = expr {
					self.push_line(&format!("local {items} = {expr}"));
				} else {
					self.push_line(&format!("local {items}"));
				}
			}

			Stmt::Assign(var, expr) => self.push_line(&format!("{var} = {expr}")),
			Stmt::Error(msg) => self.push_line(&format!("error(\"{msg}\")")),
			Stmt::Assert(cond, msg) => match msg {
				Some(msg) => self.push_line(&format!("assert({cond}, \"{msg}\")")),
				None => self.push_line(&format!("assert({cond})")),
			},

			Stmt::Call(var, method, args) => match method {
				Some(method) => self.push_line(&format!(
					"{var}:{method}({})",
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				)),

				None => self.push_line(&format!(
					"{var}({})",
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				)),
			},

			Stmt::NumFor { var, from, to } => self.push_line(&format!("for {var} = {from}, {to} do")),
			Stmt::GenFor { key, val, obj } => self.push_line(&format!("for {key}, {val} in {obj} do")),
			Stmt::If(cond) => self.push_line(&format!("if {cond} then")),
			Stmt::ElseIf(cond) => self.push_line(&format!("elseif {cond} then")),
			Stmt::Else => self.push_line("else"),

			Stmt::End => self.push_line("end"),
		};

		if matches!(
			stmt,
			Stmt::NumFor { .. } | Stmt::GenFor { .. } | Stmt::If(..) | Stmt::ElseIf(..) | Stmt::Else
		) {
			self.indent();
		};
	}

	fn push_stmts(&mut self, stmts: &[Stmt]) {
		for stmt in stmts {
			self.push_stmt(stmt);
		}
	}

	fn push_ty(&mut self, ty: &Ty) {
		if !matches!(ty, Ty::Tup(_)) {
			self.push("(");
		}

		match ty {
			Ty::Num(..) => self.push("number"),
			Ty::Str(..) => self.push("string"),
			Ty::Buf(..) => self.push("buffer"),

			Ty::Arr(ty, ..) => {
				self.push("{ ");
				self.push_ty(ty);
				self.push(" }");
			}

			Ty::Map(key, val) => {
				self.push("{ [");
				self.push_ty(key);
				self.push("]: ");
				self.push_ty(val);
				self.push(" }");
			}

			Ty::Set(key) => {
				self.push("{ [");
				self.push_ty(key);
				self.push("]: true");
				self.push(" }");
			}

			Ty::Tup(types) => {
				for (i, ty) in types.iter().enumerate() {
					self.push_ty(ty);
					if i != types.len() - 1 {
						self.push(", ");
					}
				}
			}

			Ty::Opt(ty) => {
				self.push_ty(ty);

				if !matches!(**ty, Ty::Unknown) {
					self.push("?");
				}
			}

			Ty::Ref(name) => self.push(name),

			Ty::Enum(enum_ty) => match enum_ty {
				Enum::Unit(enumerators) => self.push(
					&enumerators
						.iter()
						.map(|v| format!("\"{}\"", v))
						.collect::<Vec<_>>()
						.join(" | ")
						.to_string(),
				),

				Enum::Tagged { tag, variants } => {
					for (i, (name, struct_ty)) in variants.iter().enumerate() {
						if i != 0 {
							self.push(" | ");
						}

						self.push("{\n");
						self.indent();

						self.push_indent();

						if *name == "true" || *name == "false" {
							self.push(&format!("{tag}: {name},\n"));
						} else {
							self.push(&format!("{tag}: \"{name}\",\n"));
						}

						for (name, ty) in struct_ty.fields.iter() {
							self.push_indent();
							self.push(&format!("{name}: "));
							self.push_ty(ty);
							self.push(",\n");
						}

						self.dedent();

						self.push_indent();
						self.push("}");
					}
				}
			},

			Ty::Struct(struct_ty) => {
				self.push("{\n");
				self.indent();

				for (name, ty) in struct_ty.fields.iter() {
					self.push_indent();
					self.push(&format!("{name}: "));
					self.push_ty(ty);
					self.push(",\n");
				}

				self.dedent();
				self.push_indent();
				self.push("}");
			}

			Ty::Instance(name) => self.push(name.unwrap_or("Instance")),

			Ty::BrickColor => self.push("BrickColor"),
			Ty::DateTimeMillis => self.push("DateTime"),
			Ty::DateTime => self.push("DateTime"),
			Ty::Unknown => self.push("unknown"),
			Ty::Boolean => self.push("boolean"),
			Ty::Color3 => self.push("Color3"),
			Ty::Vector2 => self.push("Vector3"),
			Ty::Vector3 => self.push("Vector3"),
			Ty::AlignedCFrame => self.push("CFrame"),
			Ty::CFrame => self.push("CFrame"),
		}

		if !matches!(ty, Ty::Tup(_)) {
			self.push(")");
		}
	}

	fn push_file_header(&mut self, scope: &str) {
		self.push_line("--!native");
		self.push_line("--!optimize 2");
		self.push_line("--!nocheck");
		self.push_line("--!nolint");
		self.push_line("--#selene: allow(unused_variable, incorrect_standard_library_use, global_usage)");

		self.push_line(&format!(
			"-- {scope} generated by Zap v{} (https://github.com/red-blox/zap)",
			env!("CARGO_PKG_VERSION")
		));
	}
}
