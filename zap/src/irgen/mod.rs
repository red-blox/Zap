#![allow(dead_code)]
use std::{fmt::Display, vec};

use crate::config::{NumTy, Range, Ty};

pub mod des;
pub mod ser;

pub trait Gen {
	fn push_stmt(&mut self, stmt: Stmt);
	fn gen(self, var: Var, ty: &Ty<'_>) -> Vec<Stmt>;

	fn len(&mut self, buffer: Expr) {
		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("len"),
			None,
			vec!["outgoing_buff".into(), buffer],
		));
	}

	fn push_local(&mut self, name: &'static str, expr: Option<Expr>) {
		self.push_stmt(Stmt::Local(name, expr))
	}

	fn push_assign(&mut self, var: Var, expr: Expr) {
		self.push_stmt(Stmt::Assign(var, expr))
	}

	fn push_assert(&mut self, expr: Expr, msg: Option<String>) {
		self.push_stmt(Stmt::Assert(expr, msg))
	}

	fn push_writef32(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![4.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writef32"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writef64(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![8.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writef64"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writeu8(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![1.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writeu8"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writeu16(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![2.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writeu16"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writeu32(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![4.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writeu32"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writei8(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![1.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writei8"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writei16(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![2.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writei16"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writei32(&mut self, expr: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![4.0.into()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writei32"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr],
		));
	}

	fn push_writenumty(&mut self, expr: Expr, numty: NumTy) {
		match numty {
			NumTy::F32 => self.push_writef32(expr),
			NumTy::F64 => self.push_writef64(expr),
			NumTy::U8 => self.push_writeu8(expr),
			NumTy::U16 => self.push_writeu16(expr),
			NumTy::U32 => self.push_writeu32(expr),
			NumTy::I8 => self.push_writei8(expr),
			NumTy::I16 => self.push_writei16(expr),
			NumTy::I32 => self.push_writei32(expr),
		}
	}

	fn push_writestring(&mut self, expr: Expr, count: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![count.clone()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("writestring"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr, count],
		));
	}

	fn readf32(&self) -> Expr {
		Var::from("buffer")
			.nindex("readf32")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![4.0.into()])])
	}

	fn readf64(&self) -> Expr {
		Var::from("buffer")
			.nindex("readf64")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![8.0.into()])])
	}

	fn readu8(&self) -> Expr {
		Var::from("buffer")
			.nindex("readu8")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![1.0.into()])])
	}

	fn readu16(&self) -> Expr {
		Var::from("buffer")
			.nindex("readu16")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![2.0.into()])])
	}

	fn readu32(&self) -> Expr {
		Var::from("buffer")
			.nindex("readu32")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![4.0.into()])])
	}

	fn readi8(&self) -> Expr {
		Var::from("buffer")
			.nindex("readi8")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![1.0.into()])])
	}

	fn readi16(&self) -> Expr {
		Var::from("buffer")
			.nindex("readi16")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![2.0.into()])])
	}

	fn readi32(&self) -> Expr {
		Var::from("buffer")
			.nindex("readi32")
			.call(vec!["incoming_buff".into(), Var::from("read").call(vec![4.0.into()])])
	}

	fn readnumty(&self, numty: NumTy) -> Expr {
		match numty {
			NumTy::F32 => self.readf32(),
			NumTy::F64 => self.readf64(),
			NumTy::U8 => self.readu8(),
			NumTy::U16 => self.readu16(),
			NumTy::U32 => self.readu32(),
			NumTy::I8 => self.readi8(),
			NumTy::I16 => self.readi16(),
			NumTy::I32 => self.readi32(),
		}
	}

	fn readstring(&self, count: Expr) -> Expr {
		Var::from("buffer").nindex("readstring").call(vec![
			"incoming_buff".into(),
			Var::from("read").call(vec![count.clone()]),
			count,
		])
	}

	fn push_write_copy(&mut self, expr: Expr, count: Expr) {
		self.push_local("pos", Some(Var::from("alloc").call(vec![count.clone()])));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("copy"),
			None,
			vec!["outgoing_buff".into(), "pos".into(), expr, 0.0.into(), count],
		));
	}

	fn push_read_copy(&mut self, into: Var, count: Expr) {
		self.push_assign(into.clone(), Var::from("buffer").nindex("create").call(vec![count.clone()]));

		self.push_stmt(Stmt::Call(
			Var::from("buffer").nindex("copy"),
			None,
			vec![
				into.into(),
				0.0.into(),
				"incoming_buff".into(),
				Var::from("read").call(vec![count.clone()]),
				count,
			],
		));
	}

	fn push_range_check(&mut self, expr: Expr, range: Range) {
		if let Some(min) = range.min() {
			self.push_assert(expr.clone().gte(min.into()), None)
		}

		if let Some(max) = range.max() {
			self.push_assert(expr.clone().lte(max.into()), None)
		}
	}
}

#[derive(Debug, Clone)]
pub enum Stmt {
	Local(&'static str, Option<Expr>),
	Assign(Var, Expr),
	Error(String),
	Assert(Expr, Option<String>),

	Call(Var, Option<String>, Vec<Expr>),

	NumFor {
		var: &'static str,
		from: Expr,
		to: Expr,
	},
	GenFor {
		key: &'static str,
		val: &'static str,
		obj: Expr,
	},
	If(Expr),
	ElseIf(Expr),
	Else,

	End,
}

#[derive(Debug, Clone)]
pub enum Var {
	Name(String),

	NameIndex(Box<Var>, String),
	ExprIndex(Box<Var>, Box<Expr>),
}

impl Var {
	pub fn nindex(self, index: impl Into<String>) -> Self {
		Self::NameIndex(Box::new(self), index.into())
	}

	pub fn eindex(self, index: Expr) -> Self {
		Self::ExprIndex(Box::new(self), Box::new(index))
	}

	pub fn call(self, args: Vec<Expr>) -> Expr {
		Expr::Call(Box::new(self), None, args)
	}
}

impl From<&str> for Var {
	fn from(name: &str) -> Self {
		Self::Name(name.into())
	}
}

impl Display for Var {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Name(name) => write!(f, "{}", name),
			Self::NameIndex(var, index) => write!(f, "{}.{}", var, index),
			Self::ExprIndex(var, index) => write!(f, "{}[{}]", var, index),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Expr {
	// Keyword Literals
	False,
	True,
	Nil,

	// Literals
	Str(String),
	Var(Box<Var>),
	Num(f64),

	// Function Call
	Call(Box<Var>, Option<String>, Vec<Expr>),

	// Table
	EmptyTable,

	// Vector3
	Vector3(Box<Expr>, Box<Expr>, Box<Expr>),

	// Unary Operators
	Len(Box<Expr>),
	Not(Box<Expr>),

	// Boolean Binary Operators
	And(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),

	// Comparison Binary Operators
	Gte(Box<Expr>, Box<Expr>),
	Lte(Box<Expr>, Box<Expr>),
	Neq(Box<Expr>, Box<Expr>),
	Gt(Box<Expr>, Box<Expr>),
	Lt(Box<Expr>, Box<Expr>),
	Eq(Box<Expr>, Box<Expr>),

	// Arithmetic Binary Operators
	Add(Box<Expr>, Box<Expr>),
}

impl Expr {
	pub fn len(self) -> Self {
		Self::Len(Box::new(self))
	}

	pub fn not(self) -> Self {
		Self::Not(Box::new(self))
	}

	pub fn and(self, other: Self) -> Self {
		Self::And(Box::new(self), Box::new(other))
	}

	pub fn or(self, other: Self) -> Self {
		Self::Or(Box::new(self), Box::new(other))
	}

	pub fn gte(self, other: Self) -> Self {
		Self::Gte(Box::new(self), Box::new(other))
	}

	pub fn lte(self, other: Self) -> Self {
		Self::Lte(Box::new(self), Box::new(other))
	}

	pub fn neq(self, other: Self) -> Self {
		Self::Neq(Box::new(self), Box::new(other))
	}

	pub fn gt(self, other: Self) -> Self {
		Self::Gt(Box::new(self), Box::new(other))
	}

	pub fn lt(self, other: Self) -> Self {
		Self::Lt(Box::new(self), Box::new(other))
	}

	pub fn eq(self, other: Self) -> Self {
		Self::Eq(Box::new(self), Box::new(other))
	}

	pub fn add(self, other: Self) -> Self {
		Self::Add(Box::new(self), Box::new(other))
	}
}

impl From<String> for Expr {
	fn from(string: String) -> Self {
		Self::Str(string)
	}
}

impl From<Var> for Expr {
	fn from(var: Var) -> Self {
		Self::Var(Box::new(var))
	}
}

impl From<&str> for Expr {
	fn from(name: &str) -> Self {
		Self::Var(Box::new(name.into()))
	}
}

impl From<f64> for Expr {
	fn from(num: f64) -> Self {
		Self::Num(num)
	}
}

impl Display for Expr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::False => write!(f, "false"),
			Self::True => write!(f, "true"),
			Self::Nil => write!(f, "nil"),

			Self::Str(string) => write!(f, "\"{}\"", string),
			Self::Var(var) => write!(f, "{}", var),
			Self::Num(num) => write!(f, "{}", num),

			Self::Call(var, method, args) => match method {
				Some(method) => write!(
					f,
					"{}:{}({})",
					var,
					method,
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				),

				None => write!(
					f,
					"{}({})",
					var,
					args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(", ")
				),
			},

			Self::EmptyTable => write!(f, "{{}}"),

			Self::Vector3(x, y, z) => write!(f, "Vector3.new({}, {}, {})", x, y, z),

			Self::Len(expr) => write!(f, "#{}", expr),
			Self::Not(expr) => write!(f, "not {}", expr),

			Self::And(lhs, rhs) => write!(f, "{} and {}", lhs, rhs),
			Self::Or(lhs, rhs) => write!(f, "{} or {}", lhs, rhs),

			Self::Gte(lhs, rhs) => write!(f, "{} >= {}", lhs, rhs),
			Self::Lte(lhs, rhs) => write!(f, "{} <= {}", lhs, rhs),
			Self::Neq(lhs, rhs) => write!(f, "{} ~= {}", lhs, rhs),
			Self::Gt(lhs, rhs) => write!(f, "{} > {}", lhs, rhs),
			Self::Lt(lhs, rhs) => write!(f, "{} < {}", lhs, rhs),
			Self::Eq(lhs, rhs) => write!(f, "{} == {}", lhs, rhs),

			Self::Add(lhs, rhs) => write!(f, "{} + {}", lhs, rhs),
		}
	}
}
