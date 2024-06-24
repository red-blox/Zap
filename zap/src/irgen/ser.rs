use crate::config::{Enum, NumTy, Struct, Ty};

use super::{Expr, Gen, Stmt, Var};

struct Ser {
	checks: bool,
	buf: Vec<Stmt>,
}

impl Gen for Ser {
	fn push_stmt(&mut self, stmt: Stmt) {
		self.buf.push(stmt);
	}

	fn gen(mut self, var: Var, ty: &Ty) -> Vec<Stmt> {
		self.push_ty(ty, var);
		self.buf
	}
}

impl Ser {
	fn push_struct(&mut self, struct_ty: &Struct, from: Var) {
		for (name, ty) in struct_ty.fields.iter() {
			self.push_ty(ty, from.clone().nindex(*name));
		}
	}

	fn push_enum(&mut self, enum_ty: &Enum, from: Var) {
		match enum_ty {
			Enum::Unit(enumerators) => {
				let from_expr = Expr::from(from.clone());
				let numty = NumTy::from_f64(0.0, enumerators.len() as f64 - 1.0);

				for (i, enumerator) in enumerators.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(from_expr.clone().eq(Expr::Str(enumerator.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(from_expr.clone().eq(Expr::Str(enumerator.to_string()))));
					}

					self.push_writenumty((i as f64).into(), numty);
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid enumerator".into()));
				self.push_stmt(Stmt::End);
			}

			Enum::Tagged { tag, variants } => {
				let tag_expr = Expr::from(from.clone().nindex(*tag));

				for (i, variant) in variants.iter().enumerate() {
					if i == 0 {
						self.push_stmt(Stmt::If(tag_expr.clone().eq(Expr::Str(variant.0.to_string()))));
					} else {
						self.push_stmt(Stmt::ElseIf(tag_expr.clone().eq(Expr::Str(variant.0.to_string()))));
					}

					self.push_writeu8((i as f64).into());
					self.push_struct(&variant.1, from.clone());
				}

				self.push_stmt(Stmt::Else);
				self.push_stmt(Stmt::Error("Invalid variant".into()));
				self.push_stmt(Stmt::End);
			}
		}
	}

	fn push_ty(&mut self, ty: &Ty, from: Var) {
		let from_expr = Expr::from(from.clone());

		match ty {
			Ty::Num(numty, range) => {
				if self.checks {
					self.push_range_check(from_expr.clone(), *range);
				}

				self.push_writenumty(from_expr, *numty)
			}

			Ty::Str(range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_writestring(from_expr, len.into());
				} else {
					self.push_local("len", Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check("len".into(), *range);
					}

					self.push_writeu16("len".into());
					self.push_writestring(from_expr, "len".into());
				}
			}

			Ty::Buf(range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_write_copy(from_expr, len.into());
				} else {
					self.push_local(
						"len",
						Some(Var::from("buffer").nindex("len").call(vec![from_expr.clone()])),
					);

					if self.checks {
						self.push_range_check("len".into(), *range);
					}

					self.push_writeu16("len".into());
					self.push_write_copy(from_expr, "len".into())
				}
			}

			Ty::Arr(ty, range) => {
				if let Some(len) = range.exact() {
					if self.checks {
						self.push_assert(from_expr.clone().len().eq(len.into()), None);
					}

					self.push_stmt(Stmt::NumFor {
						var: "i",
						from: 1.0.into(),
						to: len.into(),
					});

					self.push_ty(ty, from.clone().eindex("i".into()));
					self.push_stmt(Stmt::End);
				} else {
					self.push_local("len", Some(from_expr.clone().len()));

					if self.checks {
						self.push_range_check("len".into(), *range);
					}

					self.push_writeu16("len".into());

					self.push_stmt(Stmt::NumFor {
						var: "i",
						from: 1.0.into(),
						to: "len".into(),
					});

					let var_name = from.display_escaped();

					self.push_stmt(Stmt::Local(
						var_name.clone().leak(),
						Some(from.clone().eindex("i".into()).into()),
					));

					self.push_ty(ty, Var::Name(var_name));
					self.push_stmt(Stmt::End);
				}
			}

			Ty::Map(key, val) => {
				self.push_local("len_pos", Some(Var::from("alloc").call(vec![2.0.into()])));
				self.push_local("len", Some(0.0.into()));

				self.push_stmt(Stmt::GenFor {
					key: "k",
					val: "v",
					obj: from_expr,
				});

				self.push_assign("len".into(), Expr::from("len").add(1.0.into()));
				self.push_ty(key, "k".into());
				self.push_ty(val, "v".into());

				self.push_stmt(Stmt::End);

				self.push_stmt(Stmt::Call(
					Var::from("buffer").nindex("writeu16"),
					None,
					vec!["outgoing_buff".into(), "len_pos".into(), "len".into()],
				));
			}

			Ty::Opt(ty) => {
				self.push_stmt(Stmt::If(from_expr.clone().eq(Expr::Nil)));

				self.push_writeu8(0.0.into());

				self.push_stmt(Stmt::Else);

				self.push_writeu8(1.0.into());
				self.push_ty(ty, from);

				self.push_stmt(Stmt::End);
			}

			Ty::Ref(name) => self.push_stmt(Stmt::Call(
				Var::from("types").nindex(format!("write_{name}")),
				None,
				vec![from_expr],
			)),

			Ty::Enum(enum_ty) => self.push_enum(enum_ty, from),
			Ty::Struct(struct_ty) => self.push_struct(struct_ty, from),

			Ty::Instance(class) => {
				if self.checks && class.is_some() {
					self.push_assert(
						Expr::Call(
							Box::new(from),
							Some("IsA".into()),
							vec![Expr::Str(class.unwrap().into())],
						),
						None,
					);
				}

				self.push_stmt(Stmt::Call(
					Var::from("table").nindex("insert"),
					None,
					vec!["outgoing_inst".into(), from_expr],
				))
			}

			Ty::Unknown => self.push_stmt(Stmt::Call(
				Var::from("table").nindex("insert"),
				None,
				vec!["outgoing_inst".into(), from_expr],
			)),

			Ty::Color3 => {
				self.push_writeu8(Expr::Mul(
					Box::new(from.clone().nindex("R").into()),
					Box::new(Expr::Num(255.0)),
				));
				self.push_writeu8(Expr::Mul(
					Box::new(from.clone().nindex("G").into()),
					Box::new(Expr::Num(255.0)),
				));
				self.push_writeu8(Expr::Mul(
					Box::new(from.clone().nindex("B").into()),
					Box::new(Expr::Num(255.0)),
				));
			}

			Ty::DateTimeMillis => {
				self.push_writef64(from.clone().nindex("UnixTimestampMillis").into());
			}

			Ty::DateTime => {
				self.push_writef64(from.clone().nindex("UnixTimestamp").into());
			}

			Ty::Vector3 => {
				self.push_writef32(from.clone().nindex("X").into());
				self.push_writef32(from.clone().nindex("Y").into());
				self.push_writef32(from.clone().nindex("Z").into());
			}

			Ty::AlignedCFrame => {
				self.push_local(
					"axis_alignment",
					Some(Expr::Call(
						Box::new(Var::from("table").nindex("find")),
						None,
						vec!["CFrameSpecialCases".into(), from.clone().nindex("Rotation").into()],
					)),
				);

				self.push_assert(
					"axis_alignment".into(),
					Some("CFrame not aligned to an axis!".to_string()),
				);

				self.push_writeu8("axis_alignment".into());

				self.push_ty(&Ty::Vector3, from.clone().nindex("Position"));
			}

			Ty::CFrame => {
				// local axis, angle = Value:ToAxisAngle()
				self.push_stmt(Stmt::LocalTuple(
					vec!["axis", "angle"],
					Some(Expr::Call(from.clone().into(), Some("ToAxisAngle".into()), vec![])),
				));

				// axis = axis * angle
				// store the angle into the axis, as it is a unit vector, so the magnitude can be used to encode a number
				self.push_stmt(Stmt::Assign(
					Var::Name("axis".into()),
					Expr::Mul(Box::new("axis".into()), Box::new("angle".into())),
				));

				self.push_ty(&Ty::Vector3, from.clone().nindex("Position"));
				self.push_ty(&Ty::Vector3, "axis".into());
			}

			Ty::Boolean => self.push_writeu8(from_expr.and(1.0.into()).or(0.0.into())),
		}
	}
}

pub fn gen(ty: &Ty, var: &str, checks: bool) -> Vec<Stmt> {
	Ser { checks, buf: vec![] }.gen(var.into(), ty)
}
