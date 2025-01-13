use crate::{
	config::{Config, EvCall, EvDecl, EvSource, EvType, FnDecl, Parameter, TyDecl, YieldType},
	irgen::{des, ser},
	output::{get_named_values, get_unnamed_values},
};

use super::Output;

struct ClientOutput<'src> {
	config: &'src Config<'src>,
	tabs: u32,
	buf: String,
}

impl<'a> Output for ClientOutput<'a> {
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
}

impl<'src> ClientOutput<'src> {
	pub fn new(config: &'src Config<'src>) -> Self {
		Self {
			config,
			tabs: 0,
			buf: String::new(),
		}
	}

	fn push_studio(&mut self) {
		self.push_line("if not RunService:IsRunning() then");
		self.indent();

		self.push_line("local noop = function() end");

		self.push_line("return table.freeze({");
		self.indent();

		let fire = self.config.casing.with("Fire", "fire", "fire");
		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let on = self.config.casing.with("On", "on", "on");
		let call = self.config.casing.with("Call", "call", "call");

		let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("{send_events} = noop,"));

		for ev in self.config.evdecls.iter() {
			self.push_line(&format!("{name} = table.freeze({{", name = ev.name));
			self.indent();

			if ev.from == EvSource::Client {
				self.push_line(&format!("{fire} = noop"));
			} else {
				match ev.call {
					EvCall::SingleSync | EvCall::SingleAsync => self.push_line(&format!("{set_callback} = noop")),
					EvCall::ManySync | EvCall::ManyAsync => self.push_line(&format!("{on} = noop")),
				}
			}

			self.dedent();
			self.push_line("}),");
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("{name} = table.freeze({{", name = fndecl.name));
			self.indent();

			self.push_line(&format!("{call} = noop"));

			self.dedent();
			self.push_line("}),");
		}

		self.dedent();
		self.push_line("}) :: Events");

		self.dedent();
		self.push_line("end");
	}

	fn push_tydecl(&mut self, tydecl: &TyDecl) {
		let name = &tydecl.name;
		let ty = &tydecl.ty;

		self.push_indent();
		self.push(&format!("export type {name} = "));
		self.push_ty(ty);
		self.push("\n");

		self.push_line(&format!("function types.write_{name}(value: {name})"));
		self.indent();
		self.push_stmts(&ser::gen(
			&[ty.clone()],
			&["value".to_string()],
			self.config.write_checks,
		));
		self.dedent();
		self.push_line("end");

		self.push_line(&format!("function types.read_{name}()"));
		self.indent();
		self.push_line("local value;");
		self.push_stmts(&des::gen(&[ty.clone()], &["value".to_string()], false));
		self.push_line("return value");
		self.dedent();
		self.push_line("end");
	}

	fn push_tydecls(&mut self) {
		for tydecl in &self.config.tydecls {
			self.push_tydecl(tydecl);
		}
	}

	fn push_event_loop_body(&mut self) {
		self.push_line("if outgoing_used ~= 0 then");
		self.indent();
		self.push_line("local buff = buffer.create(outgoing_used)");
		self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
		self.push("\n");
		self.push_line("reliable:FireServer(buff, outgoing_inst)");
		self.push("\n");
		self.push_line("outgoing_buff = buffer.create(64)");
		self.push_line("outgoing_used = 0");
		self.push_line("outgoing_size = 64");
		self.push_line("table.clear(outgoing_inst)");
		self.dedent();
		self.push_line("end");
		self.dedent();
	}

	fn push_event_loop(&mut self) {
		self.push("\n");

		let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("local function {send_events}()"));
		self.indent();
		self.push_event_loop_body();
		self.push_line("end\n");

		if !self.config.manual_event_loop {
			self.push_line(&format!("RunService.Heartbeat:Connect({send_events})\n"));
		}
	}

	fn push_reliable_header(&mut self) {
		self.push_line("reliable.OnClientEvent:Connect(function(buff, inst)");
		self.indent();
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");
		self.push_line("incoming_ipos = 0");

		self.push_line("local len = buffer.len(buff)");
		self.push_line("while incoming_read < len do");

		self.indent();

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));
	}

	fn get_values(&self, count: usize) -> String {
		if count > 0 {
			(1..=count)
				.map(|i| {
					if i == 1 {
						"value".to_string()
					} else {
						format!("value{}", i)
					}
				})
				.collect::<Vec<_>>()
				.join(", ")
		} else {
			"value".to_string()
		}
	}

	fn push_reliable_callback(&mut self, first: bool, ev: &EvDecl) {
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

		let values = self.get_values(ev.data.len());

		self.push_line(&format!("local {values}"));

		if !ev.data.is_empty() {
			self.push_stmts(&des::gen(
				ev.data.iter().map(|parameter| &parameter.ty),
				&get_unnamed_values("value", ev.data.len()),
				true,
			));
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"));
		} else {
			self.push_line(&format!("if events[{id}][1] then"));
		}

		self.indent();

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.push_line(&format!("for _, cb in events[{id}] do"));
			self.indent();
		}

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}]({values})")),
			EvCall::SingleAsync => self.push_line(&format!("1task.spawn(events[{id}], {values})")),
			EvCall::ManySync => self.push_line(&format!("cb({values})")),
			EvCall::ManyAsync => self.push_line(&format!("2task.spawn(cb, {values})")),
		}

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.dedent();
			self.push_line("end");
		}

		self.dedent();
		self.push_line("else");
		self.indent();

		if !ev.data.is_empty() {
			if ev.data.len() > 1 {
				self.push_line(&format!("table.insert(event_queue[{id}], {{ {values} }})"));
			} else {
				self.push_line(&format!("table.insert(event_queue[{id}], value)"));
			}

			self.push_line(&format!("if #event_queue[{id}] > 64 then"));
		} else {
			self.push_line(&format!("event_queue[{id}] += 1"));
			self.push_line(&format!("if event_queue[{id}] > 16 then"));
		}

		self.indent();
		self.push_indent();

		self.push("warn(`[ZAP] {");

		if !ev.data.is_empty() {
			self.push("#")
		}

		self.push(&format!(
			"event_queue[{id}]}} events in queue for {}. Did you forget to attach a listener?`)\n",
			ev.name
		));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	fn push_fn_callback(&mut self, first: bool, fndecl: &FnDecl) {
		let id = fndecl.id;

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

		let values = self.get_values(fndecl.rets.as_ref().map_or(0, |x| x.len()));

		self.push_line(&format!("local {values}"));

		if let Some(data) = &fndecl.rets {
			self.push_stmts(&des::gen(data, &get_unnamed_values("value", data.len()), true));
		}

		match self.config.yield_type {
			YieldType::Yield | YieldType::Future => {
				self.push_line(&format!("local thread = event_queue[{id}][call_id]"));
				self.push_line("-- When using actors it's possible for multiple Zap clients to exist, but only one called the Zap remote function.");
				self.push_line(&format!("if thread then"));
				self.indent();
				self.push_line(&format!("task.spawn(thread, {values})"));
				self.dedent();
				self.push_line("end");
			}
			YieldType::Promise => {
				self.push_line(&format!("event_queue[{id}][call_id]({values})"));
			}
		}

		self.push_line(&format!("event_queue[{id}][call_id] = nil"));

		self.dedent();
	}

	fn push_reliable_footer(&mut self) {
		self.push_line("else");
		self.indent();
		self.push_line("error(\"Unknown event id\")");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end)");
	}

	fn push_reliable(&mut self) {
		self.push_reliable_header();

		let mut first = true;

		for evdecl in self
			.config
			.evdecls
			.iter()
			.filter(|evdecl| evdecl.from == EvSource::Server && evdecl.evty == EvType::Reliable)
		{
			self.push_reliable_callback(first, evdecl);
			first = false;
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_fn_callback(first, fndecl);
			first = false;
		}

		self.push_reliable_footer();
	}

	fn push_unreliable_header(&mut self) {
		self.push_line("unreliable.OnClientEvent:Connect(function(buff, inst)");
		self.indent();
		self.push_line("incoming_buff = buff");
		self.push_line("incoming_inst = inst");
		self.push_line("incoming_read = 0");
		self.push_line("incoming_ipos = 0");

		self.push_line(&format!(
			"local id = buffer.read{}(buff, read({}))",
			self.config.event_id_ty(),
			self.config.event_id_ty().size()
		));
	}

	fn push_unreliable_callback(&mut self, first: bool, ev: &EvDecl) {
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

		let values = self.get_values(ev.data.len());

		self.push_line(&format!("local {values}"));

		if !ev.data.is_empty() {
			self.push_stmts(&des::gen(
				ev.data.iter().map(|parameter| &parameter.ty),
				&get_unnamed_values("value", ev.data.len()),
				self.config.write_checks,
			));
		}

		if ev.call == EvCall::SingleSync || ev.call == EvCall::SingleAsync {
			self.push_line(&format!("if events[{id}] then"));
		} else {
			self.push_line(&format!("if events[{id}][1] then"));
		}

		self.indent();

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.push_line(&format!("for _, cb in events[{id}] do"));
			self.indent();
		}

		match ev.call {
			EvCall::SingleSync => self.push_line(&format!("events[{id}]({values})")),
			EvCall::SingleAsync => self.push_line(&format!("task.spawn(events[{id}], {values})")),
			EvCall::ManySync => self.push_line(&format!("cb({values})")),
			EvCall::ManyAsync => self.push_line(&format!("task.spawn(cb, {values})")),
		}

		if ev.call == EvCall::ManySync || ev.call == EvCall::ManyAsync {
			self.dedent();
			self.push_line("end");
		}

		self.dedent();
		self.push_line("else");
		self.indent();

		if !ev.data.is_empty() {
			if ev.data.len() > 1 {
				self.push_line(&format!("table.insert(event_queue[{id}], {{ {values} }})"));
			} else {
				self.push_line(&format!("table.insert(event_queue[{id}], value)"));
			}

			self.push_line(&format!("if #event_queue[{id}] > 64 then"));
		} else {
			self.push_line(&format!("event_queue[{id}] += 1"));
			self.push_line(&format!("if event_queue[{id}] > 16 then"));
		}

		self.indent();
		self.push_indent();

		self.push("warn(`[ZAP] {");

		if !ev.data.is_empty() {
			self.push("#")
		}

		self.push(&format!(
			"event_queue[{id}]}} events in queue for {}. Did you forget to attach a listener?`)\n",
			ev.name
		));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end");

		self.dedent();
	}

	fn push_unreliable_footer(&mut self) {
		self.push_line("else");
		self.indent();
		self.push_line("error(\"Unknown event id\")");
		self.dedent();
		self.push_line("end");
		self.dedent();
		self.push_line("end)");
	}

	fn push_unreliable(&mut self) {
		self.push_unreliable_header();

		let mut first = true;

		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server && ev_decl.evty == EvType::Unreliable)
		{
			self.push_unreliable_callback(first, ev);
			first = false;
		}

		self.push_unreliable_footer();
	}

	fn push_callback_lists(&mut self) {
		self.push_line(&format!(
			"local events = table.create({})",
			self.config.evdecls.len() + self.config.fndecls.len()
		));
		self.push_line(&format!(
			"local event_queue: {{ [number]: {{ any }} }} = table.create({})",
			self.config.evdecls.len() + self.config.fndecls.len()
		));

		if !self.config.fndecls.is_empty() {
			self.push_line("local function_call_id = 0");

			if self.config.typescript && self.config.yield_type == YieldType::Promise {
				self.push_line("local Promise = _G[script].Promise")
			} else if !self.config.async_lib.is_empty() {
				self.push_line(&format!("local {} = {}", self.config.yield_type, self.config.async_lib))
			}
		}

		for evdecl in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server)
		{
			let id = evdecl.id;

			if evdecl.call == EvCall::ManyAsync || evdecl.call == EvCall::ManySync {
				self.push_line(&format!("events[{id}] = {{}}"));
			}

			if !evdecl.data.is_empty() {
				self.push_line(&format!("event_queue[{id}] = {{}}"));
			} else {
				self.push_line(&format!("event_queue[{id}] = 0"));
			}
		}

		for fndecl in self.config.fndecls.iter() {
			self.push_line(&format!("event_queue[{}] = table.create(255)", fndecl.id));
		}
	}

	fn push_write_event_id(&mut self, id: usize) {
		self.push_line(&format!("alloc({})", self.config.event_id_ty().size()));
		self.push_line(&format!(
			"buffer.write{}(outgoing_buff, outgoing_apos, {id})",
			self.config.event_id_ty()
		));
	}

	fn push_value_parameters(&mut self, parameters: &[Parameter]) {
		for (i, parameter) in parameters.iter().enumerate() {
			if i > 0 {
				self.push(", ");
			}

			if let Some(name) = parameter.name {
				self.push(&format!("{name}: "));
			} else {
				let value = format!(
					"{}{}",
					self.config.casing.with("Value", "value", "value"),
					if i == 0 { "".to_string() } else { (i + 1).to_string() }
				);

				self.push(&format!("{value}: "));
			}

			self.push_ty(&parameter.ty);
		}
	}

	fn push_return_fire(&mut self, ev: &EvDecl) {
		let fire = self.config.casing.with("Fire", "fire", "fire");
		let value = self.config.casing.with("Value", "value", "value");

		self.push_indent();
		self.push(&format!("{fire} = function("));

		if !ev.data.is_empty() {
			self.push_value_parameters(&ev.data);
		}

		self.push(")\n");
		self.indent();

		if ev.evty == EvType::Unreliable {
			self.push_line("local saved = save()");
			self.push_line("load_empty()");
		}

		self.push_write_event_id(ev.id);

		if !ev.data.is_empty() {
			self.push_stmts(&ser::gen(
				ev.data.iter().map(|parameter| &parameter.ty),
				&get_named_values(value, &ev.data),
				self.config.write_checks,
			));
		}

		if ev.evty == EvType::Unreliable {
			self.push_line("local buff = buffer.create(outgoing_used)");
			self.push_line("buffer.copy(buff, 0, outgoing_buff, 0, outgoing_used)");
			self.push_line("unreliable:FireServer(buff, outgoing_inst)");
			self.push_line("load(saved)");
		}

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_outgoing(&mut self) {
		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Client)
		{
			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			self.push_return_fire(ev);

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_queued_value(&mut self, parameters: &[Parameter]) {
		if parameters.len() > 1 {
			self.push("unpack(value)");
		} else {
			self.push("value");
		}
	}

	fn push_return_setcallback(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let set_callback = self.config.casing.with("SetCallback", "setCallback", "set_callback");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{set_callback} = function({callback}: ("));

		if !ev.data.is_empty() {
			self.push_value_parameters(&ev.data);
		}

		self.push(") -> ()): () -> ()\n");
		self.indent();

		self.push_line(&format!("events[{id}] = {callback}"));

		if !ev.data.is_empty() {
			self.push_line(&format!("for _, value in event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::SingleSync {
				self.push_indent();
				self.push(&format!("{callback}("));
				self.push_queued_value(&ev.data);
				self.push_line(")\n");
			} else {
				self.push_indent();
				self.push(&format!("task.spawn({callback}, "));
				self.push_queued_value(&ev.data);
				self.push(")\n");
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = {{}}"));
		} else {
			self.push_line(&format!("for _ = 1, event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::SingleSync {
				self.push_line(&format!("{callback}()"))
			} else {
				self.push_line(&format!("task.spawn({callback})"))
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = 0"));
		}

		self.push_line("return function()");
		self.indent();

		self.push_line(&format!("events[{id}] = nil"));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end,");
	}

	fn push_return_on(&mut self, ev: &EvDecl) {
		let id = ev.id;

		let on = self.config.casing.with("On", "on", "on");
		let callback = self.config.casing.with("Callback", "callback", "callback");

		self.push_indent();
		self.push(&format!("{on} = function({callback}: ("));

		if !ev.data.is_empty() {
			self.push_value_parameters(&ev.data);
		}

		self.push(") -> ())\n");
		self.indent();

		self.push_line(&format!("table.insert(events[{id}], {callback})"));

		if !ev.data.is_empty() {
			self.push_line(&format!("for _, value in event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::ManySync {
				self.push_indent();
				self.push(&format!("{callback}("));
				self.push_queued_value(&ev.data);
				self.push_line(")\n");
			} else {
				self.push_indent();
				self.push(&format!("task.spawn({callback}, "));
				self.push_queued_value(&ev.data);
				self.push(")\n");
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = {{}}"));
		} else {
			self.push_line(&format!("for _ = 1, event_queue[{id}] do"));
			self.indent();

			if ev.call == EvCall::ManySync {
				self.push_line(&format!("{callback}()"))
			} else {
				self.push_line(&format!("task.spawn({callback})"))
			}

			self.dedent();
			self.push_line("end");

			self.push_line(&format!("event_queue[{id}] = 0"));
		}

		self.push_line("return function()");
		self.indent();

		self.push_line(&format!(
			"table.remove(events[{id}], table.find(events[{id}], {callback}))"
		));

		self.dedent();
		self.push_line("end");

		self.dedent();
		self.push_line("end,");
	}

	pub fn push_return_listen(&mut self) {
		for ev in self
			.config
			.evdecls
			.iter()
			.filter(|ev_decl| ev_decl.from == EvSource::Server)
		{
			self.push_line(&format!("{name} = {{", name = ev.name));
			self.indent();

			match ev.call {
				EvCall::SingleSync | EvCall::SingleAsync => self.push_return_setcallback(ev),
				EvCall::ManySync | EvCall::ManyAsync => self.push_return_on(ev),
			}

			self.dedent();
			self.push_line("},");
		}
	}

	fn push_return_functions(&mut self) {
		let call = self.config.casing.with("Call", "call", "call");
		let value = self.config.casing.with("Value", "value", "value");

		for fndecl in self.config.fndecls.iter() {
			let id = fndecl.id;

			self.push_line(&format!("{name} = {{", name = fndecl.name));
			self.indent();

			self.push_indent();
			self.push(&format!("{call} = function("));

			if !fndecl.args.is_empty() {
				self.push_value_parameters(&fndecl.args);
			}

			self.push(")");

			if let Some(types) = &fndecl.rets {
				match self.config.yield_type {
					YieldType::Future => {
						self.push(": Future.Future<(");

						for (i, ty) in types.iter().enumerate() {
							if i > 0 {
								self.push(", ");
							}
							self.push_ty(ty);
						}

						self.push(")>");
					}
					YieldType::Yield => {
						self.push(": (");
						for (i, ty) in types.iter().enumerate() {
							if i > 0 {
								self.push(", ");
							}
							self.push_ty(ty);
						}
						self.push(")");
					}
					_ => (),
				}
			}

			self.push("\n");
			self.indent();

			self.push_write_event_id(fndecl.id);

			self.push_line("function_call_id += 1");

			self.push_line("function_call_id %= 256");

			self.push_line(&format!("if event_queue[{id}][function_call_id] then"));
			self.indent();

			self.push_line("function_call_id -= 1");
			self.push_line("error(\"Zap has more than 256 calls awaiting a response, and therefore this packet has been dropped\")");

			self.dedent();
			self.push_line("end");

			self.push_line("alloc(1)");
			self.push_line("buffer.writeu8(outgoing_buff, outgoing_apos, function_call_id)");

			if !fndecl.args.is_empty() {
				self.push_stmts(&ser::gen(
					fndecl.args.iter().map(|parameter| &parameter.ty),
					&get_named_values(value, &fndecl.args),
					self.config.write_checks,
				));
			}

			match self.config.yield_type {
				YieldType::Yield => {
					self.push_line(&format!("event_queue[{id}][function_call_id] = coroutine.running()",));
					self.push_line("return coroutine.yield()");
				}
				YieldType::Future => {
					self.push_line("return Future.new(function()");
					self.indent();

					self.push_line(&format!("event_queue[{id}][function_call_id] = coroutine.running()",));
					self.push_line("return coroutine.yield()");

					self.dedent();
					self.push_line("end)");
				}
				YieldType::Promise => {
					self.push_line("return Promise.new(function(resolve)");
					self.indent();

					self.push_line(&format!("event_queue[{id}][function_call_id] = resolve"));

					self.dedent();
					self.push_line("end)");
				}
			}

			self.dedent();
			self.push_line("end,");

			self.dedent();
			self.push_line("},");
		}
	}

	pub fn push_return(&mut self) {
		self.push_line("local returns = {");
		self.indent();

		let send_events = self.config.casing.with("SendEvents", "sendEvents", "send_events");

		self.push_line(&format!("{send_events} = {send_events},"));

		self.push_return_outgoing();
		self.push_return_listen();
		self.push_return_functions();

		self.dedent();
		self.push_line("}");

		self.push_line("type Events = typeof(returns)");
		self.push_line("return returns");
	}

	pub fn push_remotes(&mut self) {
		self.push_line(&format!(
			"local remotes = ReplicatedStorage:WaitForChild(\"{}\")",
			self.config.remote_folder
		));
		self.push_line(&format!(
			"local reliable = remotes:WaitForChild(\"{}_RELIABLE\")",
			self.config.remote_scope
		));
		self.push_line(&format!(
			"local unreliable = remotes:WaitForChild(\"{}_UNRELIABLE\")",
			self.config.remote_scope
		));
		self.push("\n");
		self.push_line(&format!(
			"assert(reliable:IsA(\"RemoteEvent\"), \"Expected {}_RELIABLE to be a RemoteEvent\")",
			self.config.remote_scope
		));
		self.push_line(&format!("assert(unreliable:IsA(\"UnreliableRemoteEvent\"), \"Expected {}_UNRELIABLE to be an UnreliableRemoteEvent\")", self.config.remote_scope));
		self.push("\n");
	}

	pub fn push_check_server(&mut self) {
		self.push_line("if RunService:IsServer() then");
		self.indent();
		self.push_line("error(\"Cannot use the client module on the server!\")");
		self.dedent();
		self.push_line("end");
	}

	pub fn output(mut self) -> String {
		self.push_file_header("Client");

		if self.config.evdecls.is_empty() && self.config.fndecls.is_empty() {
			return self.buf;
		};

		self.push(include_str!("base.luau"));

		self.push_studio();

		self.push_check_server();

		self.push_remotes();

		self.push_tydecls();

		self.push_event_loop();

		self.push_callback_lists();

		if !self.config.fndecls.is_empty()
			|| self
				.config
				.evdecls
				.iter()
				.any(|ev| ev.evty == EvType::Reliable && ev.from == EvSource::Server)
		{
			self.push_reliable();
		}

		if self
			.config
			.evdecls
			.iter()
			.any(|ev| ev.evty == EvType::Unreliable && ev.from == EvSource::Server)
		{
			self.push_unreliable();
		}

		self.push_return();

		self.buf
	}
}

pub fn code(config: &Config) -> String {
	ClientOutput::new(config).output()
}
