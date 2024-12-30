use std::fmt::Display;

trait Logger {
    fn op_started(
}

/// Several types of entities are reported on, such as the engine, or individual units
#[derive(PartialEq, Debug, Clone)]
enum EntityHeader {
    None,
    LoadUnits,
    Engine,
    Unit(UnitArc),
    Op(UnitArc, Operation),
}

impl fmt::Display for EntityHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntityHeader::None => Ok(()),
            EntityHeader::LoadUnits => write!(f, "[ Load Units ]"),
            EntityHeader::Engine => write!(f, "[Engine]"),
            EntityHeader::Unit(unit) => write!(f, "[ Unit | {} ]", render_unit(unit)),
            EntityHeader::Op(unit, operation) => write!(f, "[ {} | {} ]", operation, render_unit(unit)),
        }
    }
}

//! Allows for tree-like logs with nested section headers
pub struct LoggingTree<H: Display + PartialEq + Clone> {
    header_stack: Vec<H>,
}

impl <H: Display + PartialEq + Clone> LoggingTree<H> {
    pub fn new() -> LoggingTree<H> {
        LoggingTree { header_stack: Vec::new() }
    }

    pub fn push_header(&mut self, header: H) {
        self.output(&header);
        self.header_stack.push(header);
    }

    pub fn pop_header(&mut self) {
        self.header_stack.pop();
    }

    pub fn set_last(&mut self, header: H) {
        match self.header_stack.last() {
            Some(last) => {
                if last != &header {
                    self.output("");
                    self.output(&header.to_string());
                }
            },
            None => {
                self.output(&header.to_string());
            }
        }
    }

    pub fn output(&self, line: &str) {
        for _ in &self.header_stack {
            print!("  ");
        }
        println!("{}", line);
    }

    pub fn print(&self, txt: &str) {
        println!("{}", txt);
    }

    pub fn print_indent(&self, txt: &str) {
        for _ in &self.header_stack {
            print!("  ");
        }
        print!("{}", txt);
    }
}

fn render_deps(deps: &Vec<Dependency>) -> String {
    deps.iter().map(|dep| format!("{}", render_dep(dep))).collect::<Vec<String>>().join(", ")
}

fn render_dep(dep: &Dependency) -> String {
    format!("{}({})", dep.name, render_valset(&dep.args))
}

fn truncate_str(s: &str, len: usize) -> String {
    if s.len() > len {
        format!("{}...", &s[..len])
    } else {
        s.to_string()
    }
}

fn render_valset(valset: &ValueSet) -> String {
    valset
        .values
        .iter()
        .map(|(key, val)| format!("{}: {}", key, truncate_str(&val.to_string(), 10)))
        .collect::<Vec<String>>()
        .join(", ")
}

fn render_meta(meta: &Meta) -> String {
    let params_str = meta.params.iter().map(|param| render_param(&param)).collect::<Vec<String>>().join(", ");
    format!("Meta Params: [{}]", params_str)
}

fn render_unit(unit: &UnitArc) -> String {
    format!("Unit: {}({})", unit.name, render_valset(&unit.args))
}

fn render_param(param: &Param) -> String {
    format!("{}:{} (required: {})", param.name, param.value_type, param.required)
}
