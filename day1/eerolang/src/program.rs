use std::{collections::HashMap, io::Write, rc::Rc};

use log::trace;

use crate::{
    ast_parser::AstNode,
    tokenizer::{Operator, Value},
};

fn builtin_print(args: &mut [Value]) -> Option<Value> {
    let mut w = std::io::stdout();
    for (i, arg) in args.iter().enumerate() {
        match arg {
            Value::Integer(i) => write!(&mut w, "{}", i).unwrap(),
            Value::Float(f) => write!(&mut w, "{}", f).unwrap(),
            Value::String(s) => write!(&mut w, "\"{}\"", s).unwrap(),
            Value::List(l) => {
                write!(&mut w, "[").unwrap();
                for (j, item) in l.iter().enumerate() {
                    match item {
                        Value::Integer(ii) => write!(&mut w, "{}", ii).unwrap(),
                        Value::Float(ff) => write!(&mut w, "{}", ff).unwrap(),
                        Value::String(ss) => write!(&mut w, "\"{}\"", ss).unwrap(),
                        Value::List(_) => write!(&mut w, "<nested list>").unwrap(),
                    }
                    if j < l.len() - 1 {
                        print!(", ");
                    }
                }
                write!(&mut w, "]").unwrap();
            }
        }
        if i < args.len() - 1 {
            write!(&mut w, " ").unwrap();
        }
    }
    writeln!(&mut w).unwrap();
    w.flush().unwrap();
    None
}

fn builtin_readfile(args: &mut [Value]) -> Option<Value> {
    assert_eq!(args.len(), 1);

    let [Value::String(filename)] = &args else {
        panic!("readfile expects (string), got {:?}", args)
    };

    let content = std::fs::read_to_string(filename.as_ref())
        .unwrap_or_else(|_| panic!("Failed to read file: {}", filename));

    Some(Value::String(content.trim().into()))
}

fn builtin_split(args: &mut [Value]) -> Option<Value> {
    let [Value::String(s), Value::String(delim)] = &args else {
        panic!("split expects (string, string), got {:?}", args)
    };

    trace!("Splitting string '{}' by delimiter '{}'", s, delim);

    let parts: Vec<Value> = s
        .split(delim.as_ref())
        .map(|part| Value::String(Rc::from(part)))
        .collect();

    Some(Value::List(parts))
}

fn builtin_len(args: &mut [Value]) -> Option<Value> {
    assert_eq!(args.len(), 1, "len expects 1 argument");

    match &args[0] {
        Value::String(s) => Some(Value::Integer(s.len() as i64)),
        Value::List(l) => Some(Value::Integer(l.len() as i64)),
        _ => panic!("len expects (string) or (list), got {:?}", args),
    }
}

pub type ProgramFn = fn(&mut [Value]) -> Option<Value>;

pub struct Program {
    block: Rc<Vec<AstNode>>,
    vars: HashMap<String, Value>,
    builtins: HashMap<String, ProgramFn>,
}

impl Program {
    pub fn new(block: Vec<AstNode>) -> Self {
        let mut builtins = HashMap::<String, ProgramFn>::new();
        builtins.insert("print".to_owned(), builtin_print);
        builtins.insert("readfile".to_owned(), builtin_readfile);
        builtins.insert("split".to_owned(), builtin_split);
        builtins.insert("len".to_owned(), builtin_len);

        Program {
            block: Rc::new(block),
            vars: HashMap::new(),
            builtins,
        }
    }

    fn compute_expression<'a>(&'a self, expr: &'a AstNode) -> Value {
        match expr {
            AstNode::Literal(lit) => lit.clone(),
            AstNode::Variable(name) => self.vars.get(name).expect("Undefined variable").clone(),
            AstNode::FunctionCall(name, args) => self
                .call_function(name, args)
                .expect("Function did not return a value"),
            AstNode::List(list) => {
                let values = list
                    .iter()
                    .map(|elem| self.compute_expression(elem))
                    .collect::<Vec<_>>();
                Value::List(values)
            }
            AstNode::BinaryOp(left, op, right) => {
                let mut left_val = self.compute_expression(left);
                let mut right_val = self.compute_expression(right);
                if let (Value::String(l), Value::String(r), Operator::Plus) =
                    (&left_val, &right_val, op)
                {
                    return Value::String(Rc::from([l.as_ref(), r.as_ref()].concat()));
                }

                if let (Value::Integer(l), Value::Integer(r)) = (&left_val, &right_val) {
                    match op {
                        Operator::Plus => return Value::Integer(l + r),
                        Operator::Minus => return Value::Integer(l - r),
                        Operator::Multiply => return Value::Integer(l * r),
                        Operator::Divide => return Value::Integer(l / r),
                    }
                }

                // Promote to float if both weren't integers
                if let Value::Integer(i) = &right_val {
                    right_val = Value::Float(*i as f64);
                };
                if let Value::Integer(i) = &left_val {
                    left_val = Value::Float(*i as f64);
                };

                if let (Value::Float(l), Value::Float(r)) = (left_val, right_val) {
                    return match op {
                        Operator::Plus => Value::Float(l + r),
                        Operator::Minus => Value::Float(l - r),
                        Operator::Multiply => Value::Float(l * r),
                        Operator::Divide => Value::Float(l / r),
                    };
                }

                panic!("Unsupported operand types for binary operation");
            }
            _ => panic!("Unsupported expression type"),
        }
    }

    fn call_function(&self, name: &str, args: &[AstNode]) -> Option<Value> {
        let mut arg_values = args
            .iter()
            .map(|arg| self.compute_expression(arg))
            .collect::<Vec<_>>();
        if let Some(func) = self.builtins.get(name) {
            func(&mut arg_values)
        } else {
            panic!("Undefined function: {}", name);
        }
    }

    pub fn execute(&mut self) {
        let block = Rc::clone(&self.block);
        for node in block.iter() {
            match node {
                AstNode::Assign(var, expr) => {
                    trace!("Assigning to variable: {}", var);
                    let value = self.compute_expression(expr);
                    self.vars.insert(var.clone(), value.clone());
                }
                AstNode::FunctionCall(name, args) => {
                    trace!("Calling function: {}", name);
                    self.call_function(name, args);
                }
                n => {
                    self.compute_expression(n);
                    panic!("Unexpected AST node during execution: {:#?}", node);
                }
            }
        }
    }
}
