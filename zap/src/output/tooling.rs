use crate::{
	config::{Config, EvDecl, FnDecl, TyDecl},
	irgen::{des, Stmt},
	Output,
};

struct ToolingOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'src> ToolingOutput<'src> {
	pub fn new(config: &'src Config) -> Self {
		Self {
			config,
			tabs: 0,
			buf: String::new(),
		}
	}

	fn push(&mut self, s: &str) {
		self.buf.push_str(s);
	}

	fn indent(&mut self) {
		self.tabs += 1;
	}

	fn dedent(&mut self) {
		self.tabs -= 1;
	}

	fn push_indent(&mut self) {
		for _ in 0..self.tabs {
			self.push("\t");
		}
	}

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

	fn push_tydecl(&mut self, tydecl: &TyDecl) {
		let name = &tydecl.name;
		let ty = &tydecl.ty;

		self.push_line(&format!("function types.read_{name}()"));
		self.indent();
		self.push_line("local value;");
		self.push_stmts(&des::gen(ty, "value", true));
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		self.push_line("local types = {}");

		for tydecl in self.config.tydecls.iter() {
			self.push_tydecl(tydecl);
		}
	}

	fn push_event_callback(&mut self, first: bool, ev: &EvDecl) {
		let id = ev.id;

		self.push_indent();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		// push_line is not used here as indent was pushed above
		// and we don't want to push it twice, especially after
		// the if/elseif
		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.indent();

		self.push_line("local value");

		if let Some(data) = &ev.data {
			self.push_stmts(&des::gen(data, "value", true));
		}

		self.push_line("table.insert(events, {");
		self.indent();

		self.push_line(&format!("Name = \"{}\",", ev.name));

		self.push_indent();
		self.push("Arguments = { ");

		if self.config.tooling_show_internal_data {
			self.push(&format!(
				"{{ {} = id }}, ",
				self.config.casing.with("EventId", "eventId", "event_id")
			));
		}

		self.push("value }");
		self.push("\n");

		self.dedent();
		self.push_line("})");

		self.dedent();
	}

	fn push_function_callback(&mut self, first: bool, fn_decl: &FnDecl) {
		let id = fn_decl.id;
		let event_id = self.config.casing.with("EventId", "eventId", "event_id");
		let call_id = self.config.casing.with("CallId", "callId", "call_id");

		self.push_indent();

		if first {
			self.push("if ");
		} else {
			self.push("elseif ");
		}

		// push_line is not used here as indent was pushed above
		// and we don't want to push it twice, especially after
		// the if/elseif
		self.push(&format!("id == {id} then"));
		self.push("\n");

		self.indent();

		self.push_line("local call_id = buffer.readu8(incoming_buff, read(1))");

		self.push_line("if isServer then");
		self.indent();

		self.push_line("local value");

		if let Some(data) = &fn_decl.args {
			self.push_stmts(&des::gen(data, "value", true));
		}

		self.push_line("table.insert(events, {");
		self.indent();

		self.push_line(&format!("Name = \"{} (request)\",", fn_decl.name));

		self.push_indent();
		self.push("Arguments = { ");

		if self.config.tooling_show_internal_data {
			self.push(&format!("{{ {} = id, {} = call_id }}, ", event_id, call_id));
		}

		self.push("value }");
		self.push("\n");

		self.dedent();
		self.push_line("})");

		self.dedent();
		self.push_line("else");
		self.indent();

		self.push_line("local value");

		if let Some(data) = &fn_decl.rets {
			self.push_stmts(&des::gen(data, "value", true));
		}

		self.push_line("table.insert(events, {");
		self.indent();

		self.push_line(&format!("Name = \"{} (callback)\",", fn_decl.name));

		self.push_indent();
		self.push("Arguments = { ");

		if self.config.tooling_show_internal_data {
			self.push(&format!("{{ {} = id, {} = call_id }}, ", event_id, call_id));
		}

		self.push("value }");
		self.push("\n");

		self.dedent();
		self.push_line("})");

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	pub fn output(mut self) -> String {
		self.push_line("--!native");
		self.push_line("--!optimize 2");
		self.push_line("--!nocheck");
		self.push_line("--!nolint");
		self.push_line("--#selene: allow(unused_variable, incorrect_standard_library_use)");

		self.push_line(&format!(
			"-- Tooling generated by Zap v{} (https://github.com/red-blox/zap)",
			env!("CARGO_PKG_VERSION")
		));

		// if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
		// 	return self.buf;
		// };

		self.push_line("local ReplicatedStorage = game:GetService(\"ReplicatedStorage\")");
		self.push("\n");

		self.push_line("return function(remote_instance, player, incoming_buff, incoming_inst)");
		self.indent();

		self.push_line(&format!(
			"local reliable = ReplicatedStorage:FindFirstChild(\"{}_RELIABLE\")",
			self.config.remote_scope
		));
		self.push_line(&format!(
			"local unreliable = ReplicatedStorage:FindFirstChild(\"{}_UNRELIABLE\")",
			self.config.remote_scope
		));
		self.push("\n");

		self.push_line("if not reliable or not unreliable then");
		self.indent();

		self.push_line("return");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line("if remote_instance ~= reliable and remote_instance ~= unreliable then");
		self.indent();

		self.push_line("return");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line("local isServer = true");
		self.push_line("if type(player) == \"buffer\" then");
		self.indent();

		self.push_line("isServer = false");
		self.push_line("incoming_inst = incoming_buff");
		self.push_line("incoming_buff = player");
		self.push_line("player = nil");

		self.dedent();
		self.push_line("end");
		self.push("\n");

		self.push_line("local incoming_read = 0");
		self.push_line("local incoming_ipos = 0");

		self.push_line("local function read(len: number)");
		self.indent();

		self.push_line("local pos = incoming_read");
		self.push_line("incoming_read = incoming_read + len");
		self.push("\n");
		self.push_line("return pos");

		self.dedent();
		self.push_line("end");
		self.push_line("local len = buffer.len(incoming_buff)");
		self.push("\n");

		self.push_tydecls();
		self.push("\n");

		self.push_line("local events = {}");
		self.push_line("while incoming_read < len do");

		self.indent();

		self.push_line(&format!(
			"local id = buffer.read{}(incoming_buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));

		let mut first = true;

		for ev in self.config.evdecls.iter() {
			self.push_event_callback(first, ev);

			first = false;
		}

		for fn_decl in self.config.fndecls.iter() {
			self.push_function_callback(first, fn_decl);

			first = false;
		}

		self.push_line("else");
		self.indent();
		self.push_line("error(\"Unknown event id\")");

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end");

		self.push("\n");
		self.push_line("return events");

		self.dedent();
		self.push_line("end");

		self.buf
	}
}

pub fn output(config: &Config) -> Option<Output> {
	if !config.tooling {
		return None;
	}

	#[cfg(not(target_arch = "wasm32"))]
	let output = Output {
		code: ToolingOutput::new(config).output(),
		defs: None,
		path: config.tooling_output.into(),
	};

	#[cfg(target_arch = "wasm32")]
	let output = Output {
		code: ToolingOutput::new(config).output(),
		defs: None,
	};

	Some(output)
}
