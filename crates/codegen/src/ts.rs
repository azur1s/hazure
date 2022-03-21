use std::fmt::Display;

use hir::{IR, IRKind, Value};

pub struct Codegen {
    pub emitted: String,
}

impl Codegen {
    pub fn new() -> Self {
        Self { emitted: String::new() }
    }

    fn emit<T: Display>(&mut self, t: T) {
        self.emitted.push_str(&t.to_string());
    }

    pub fn gen(&mut self, irs: Vec<IR>) {
        self.emit(format!("// Auto-generated by hazure compiler version {}\n", env!("CARGO_PKG_VERSION")));
        self.emit("import { read, write, readFile, writeFile } from \"https://raw.githubusercontent.com/azur1s/hazure/master/runtime/io.ts\"\n");

        for ir in irs {
            self.emit(&self.gen_ir(&ir.kind, true));
        }

        self.emit("f_main();");
    }

    fn gen_ir(&self, ir: &IRKind, should_gen_semicolon: bool) -> String {
        #[macro_export]
        macro_rules! semicolon { () => { if should_gen_semicolon { ";" } else { "" } }; }

        match ir {
            IRKind::Define { public, name, type_hint, value, mutable } => {
                format!(
                    "{} {} v_{}: {} = {}{}\n",
                    if *public { "export" } else { "" },
                    if *mutable { "let" } else { "const" },
                    name, 
                    type_hint,
                    self.gen_ir(value, false),
                    semicolon!()
                )
            },

            IRKind::Call { name, args } => {
                format!(
                    "f_{}({}){}",
                    name,
                    args
                        .iter()
                        .map(|arg| self.gen_ir(arg, false))
                        .collect::<Vec<_>>()
                        .join(", ")
                        .trim_end_matches(";\n"),
                    semicolon!(),
                )
            },

            IRKind::Intrinsic { name, args } => {
                match name.as_str() {
                    "write"      => { format!("write({}){}\n"        , self.gen_ir(&args[0], false), semicolon!()) },
                    "write_file" => { format!("writeFile({}, {}){}\n", self.gen_ir(&args[0], false), self.gen_ir(&args[1], false), semicolon!()) },
                    "read"       => { format!("read({}){}\n"         , self.gen_ir(&args[0], false), semicolon!()) },
                    "read_file"  => { format!("readFile({}){}\n"     , self.gen_ir(&args[0], false), semicolon!()) }
                    "emit" => { format!("{}", self.gen_ir(&args[0], false).trim_start_matches('"').trim_end_matches('"')) },
                    "get" => { format!("{}[{}]", self.gen_ir(&args[0], false), self.gen_ir(&args[1], false)) },
                    _ => unreachable!(format!("Unknown intrinsic: {}", name)) // Shoul be handled by lowering
                }
            },

            IRKind::Fun { public, name, return_type_hint, args, body } => {
                let args = args
                    .iter()
                    .map(|arg| format!("v_{}: {}", arg.0, arg.1))
                    .collect::<Vec<_>>().
                    join(", ");
                format!(
                    "{} const f_{} = ({}): {} => {};\n",
                    if *public { "export" } else { "" },
                    name,
                    args,
                    return_type_hint,
                    self.gen_ir(body, false)
                )
            },

            IRKind::Return { value } => {
                format!(
                    "return {};\n",
                    self.gen_ir(value, false)
                )
            },

            IRKind::Do { body } => {
                let mut out = "{\n".to_string();
                for expr in body {
                    out.push_str(&self.gen_ir(&expr, true));
                }
                out.push_str("}\n");
                out
            },

            IRKind::If { cond, body, else_body } => {
                format!(
                    "if ({}) {{\n{}}} else {{\n{}}}\n",
                    self.gen_ir(cond, true),
                    self.gen_ir(body, true),
                    self.gen_ir(else_body, true),
                )
            },

            IRKind::Case { cond, cases, default } => {
                format!(
                    "switch ({}) {{\n{}{}\n}}\n",
                    self.gen_ir(cond, true),
                    cases
                        .iter()
                        .map(|(pattern, body)| format!(
                            "case {}: {}\nbreak;\n",
                            self.gen_ir(pattern, true),
                            self.gen_ir(body, true)))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    format!(
                        "default: {}\nbreak;\n",
                        self.gen_ir(default, true),
                    ),
                )
            },

            IRKind::Unary { op, right } => {
                format!("{}{}", op, self.gen_ir(right, false))
            },

            IRKind::Binary { left, op, right } => {
                format!("{} {} {}", self.gen_ir(left, false), op, self.gen_ir(right, false))
            },

            IRKind::Value { value } => {
                match value {
                    Value::Int(value)     => format!("{}", value),
                    Value::Boolean(value) => format!("{}", value),
                    Value::String(value)  => format!("\"{}\"", value),
                    Value::Ident(value)   => format!("v_{}", value),
                }
            },

            IRKind::Vector { values } => {
                format!(
                    "[{}]",
                    values
                        .iter()
                        .map(|value| self.gen_ir(value, false))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },

            #[allow(unreachable_patterns)]
            _ => { dbg!(ir); todo!() },
        }
    }
}
