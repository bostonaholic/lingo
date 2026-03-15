/// Tree-walking evaluator for the Lingo programming language.

use std::collections::HashMap;
use std::fmt;

use crate::ast::*;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
    Tuple(Vec<Value>),
    List(Vec<Value>),
    Fn {
        name: String,
        params: Vec<Param>,
        body: Block,
        closure: Env,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
        closure: Env,
    },
    BuiltinFn(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Unit => write!(f, "()"),
            Value::Tuple(vals) => {
                write!(f, "(")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::List(vals) => {
                write!(f, "[")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Fn { name, .. } => write!(f, "<fn {}>", name),
            Value::Lambda { .. } => write!(f, "<lambda>"),
            Value::BuiltinFn(name) => write!(f, "<builtin {}>", name),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Unit => false,
            Value::List(l) => !l.is_empty(),
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn child(parent: &Env) -> Self {
        Env {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.bindings.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn update(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &mut self.parent {
            parent.update(name, value)
        } else {
            false
        }
    }
}

/// Control flow signals
enum Signal {
    Return(Value),
    Break,
}

pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Env::new();

        // Register built-in functions
        let builtins = vec![
            "println", "print", "to_str", "to_int", "to_float", "len", "push",
            "map", "filter", "fold", "range", "split", "join", "trim",
            "contains", "sort", "sort_by", "rev", "enumerate", "zip",
            "flat_map", "any", "all", "find", "unique", "chunk", "take",
            "skip", "min", "max", "abs", "dbg", "assert", "type_of",
            "read_file", "write_file", "read_line", "parse_json",
            "group_by", "flatten", "reduce", "replace", "starts_with",
            "ends_with", "to_upper", "to_lower",
        ];
        for name in builtins {
            env.set(name.to_string(), Value::BuiltinFn(name.to_string()));
        }

        Interpreter { env }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        // First pass: collect all function declarations
        for item in &program.items {
            if let Item::FnDecl(f) = item {
                let val = Value::Fn {
                    name: f.name.clone(),
                    params: f.params.clone(),
                    body: f.body.clone(),
                    closure: self.env.clone(),
                };
                self.env.set(f.name.clone(), val);
            }
        }

        // Look for main() and call it
        if let Some(main_fn) = self.env.get("main") {
            self.call_function(&main_fn, &[])?;
        } else {
            // Execute top-level expressions
            for item in &program.items {
                if let Item::ExprStmt(expr) = item {
                    self.eval_expr(expr)?;
                }
            }
        }

        Ok(())
    }

    fn eval_block(&mut self, block: &Block) -> Result<Value, String> {
        let saved_env = self.env.clone();
        self.env = Env::child(&saved_env);

        let result = self.eval_block_inner(block);

        self.env = saved_env;
        result
    }

    fn eval_block_inner(&mut self, block: &Block) -> Result<Value, String> {
        for stmt in &block.stmts {
            match self.eval_stmt(stmt) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        if let Some(expr) = &block.expr {
            self.eval_expr(expr)
        } else {
            Ok(Value::Unit)
        }
    }

    fn eval_block_with_signals(&mut self, block: &Block) -> Result<Value, Result<Signal, String>> {
        for stmt in &block.stmts {
            match self.eval_stmt_with_signals(stmt) {
                Ok(_) => {}
                Err(Ok(signal)) => return Err(Ok(signal)),
                Err(Err(e)) => return Err(Err(e)),
            }
        }

        if let Some(expr) = &block.expr {
            self.eval_expr(expr).map_err(Err)
        } else {
            Ok(Value::Unit)
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, String> {
        match self.eval_stmt_with_signals(stmt) {
            Ok(v) => Ok(v),
            Err(Ok(Signal::Return(v))) => Err(format!("__return__{}", self.value_to_debug(&v))),
            Err(Ok(Signal::Break)) => Err("__break__".to_string()),
            Err(Err(e)) => Err(e),
        }
    }

    fn eval_stmt_with_signals(&mut self, stmt: &Stmt) -> Result<Value, Result<Signal, String>> {
        match stmt {
            Stmt::Let(let_stmt) => {
                let value = self.eval_expr(&let_stmt.value).map_err(Err)?;
                self.bind_pattern(&let_stmt.pattern, &value).map_err(Err)?;
                Ok(Value::Unit)
            }
            Stmt::Expr(expr) => self.eval_expr(expr).map_err(Err),
            Stmt::For(for_stmt) => {
                let iterable = self.eval_expr(&for_stmt.iterable).map_err(Err)?;
                let items = self.value_to_iterable(&iterable).map_err(Err)?;

                let saved_env = self.env.clone();

                for item in items {
                    self.env = Env::child(&saved_env);
                    self.bind_pattern(&for_stmt.binding, &item).map_err(Err)?;
                    match self.eval_block_with_signals(&for_stmt.body) {
                        Ok(_) => {}
                        Err(Ok(Signal::Break)) => {
                            self.env = saved_env;
                            return Ok(Value::Unit);
                        }
                        Err(Ok(Signal::Return(v))) => {
                            self.env = saved_env;
                            return Err(Ok(Signal::Return(v)));
                        }
                        Err(Err(e)) => {
                            self.env = saved_env;
                            return Err(Err(e));
                        }
                    }
                }
                self.env = saved_env;
                Ok(Value::Unit)
            }
            Stmt::While(while_stmt) => {
                loop {
                    let cond = self.eval_expr(&while_stmt.condition).map_err(Err)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    let saved_env = self.env.clone();
                    self.env = Env::child(&saved_env);
                    match self.eval_block_with_signals(&while_stmt.body) {
                        Ok(_) => {}
                        Err(Ok(Signal::Break)) => {
                            self.env = saved_env;
                            break;
                        }
                        Err(Ok(Signal::Return(v))) => {
                            self.env = saved_env;
                            return Err(Ok(Signal::Return(v)));
                        }
                        Err(Err(e)) => {
                            self.env = saved_env;
                            return Err(Err(e));
                        }
                    }
                    self.env = saved_env;
                }
                Ok(Value::Unit)
            }
            Stmt::Return(maybe_expr) => {
                let val = if let Some(expr) = maybe_expr {
                    self.eval_expr(expr).map_err(Err)?
                } else {
                    Value::Unit
                };
                Err(Ok(Signal::Return(val)))
            }
            Stmt::Break => Err(Ok(Signal::Break)),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_to_value(lit)),
            Expr::Ident(name, _span) => {
                self.env
                    .get(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expr::Binary(left, op, right, _span) => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binop(op, &l, &r)
            }
            Expr::Unary(op, operand, _span) => {
                let val = self.eval_expr(operand)?;
                self.eval_unop(op, &val)
            }
            Expr::Call(callee, args, _span) => {
                let func = self.eval_expr(callee)?;
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }
                self.call_function(&func, &eval_args)
            }
            Expr::Pipeline(left, right, _span) => {
                let left_val = self.eval_expr(left)?;
                // Desugar: x |> f(a, b) => f(x, a, b)
                //          x |> f       => f(x)
                match right.as_ref() {
                    Expr::Call(callee, args, _span) => {
                        let func = self.eval_expr(callee)?;
                        let mut all_args = vec![left_val];
                        for arg in args {
                            all_args.push(self.eval_expr(arg)?);
                        }
                        self.call_function(&func, &all_args)
                    }
                    Expr::Ident(_, _) => {
                        let func = self.eval_expr(right)?;
                        self.call_function(&func, &[left_val])
                    }
                    _ => {
                        let func = self.eval_expr(right)?;
                        self.call_function(&func, &[left_val])
                    }
                }
            }
            Expr::Index(obj, idx, _span) => {
                let obj_val = self.eval_expr(obj)?;
                let idx_val = self.eval_expr(idx)?;
                match (&obj_val, &idx_val) {
                    (Value::List(list), Value::Int(i)) => {
                        let i = *i as usize;
                        list.get(i)
                            .cloned()
                            .ok_or_else(|| format!("Index {} out of bounds (len {})", i, list.len()))
                    }
                    (Value::Str(s), Value::Int(i)) => {
                        let i = *i as usize;
                        s.chars()
                            .nth(i)
                            .map(|c| Value::Str(c.to_string()))
                            .ok_or_else(|| format!("Index {} out of bounds", i))
                    }
                    _ => Err(format!("Cannot index {:?} with {:?}", obj_val, idx_val)),
                }
            }
            Expr::Field(obj, field, _span) => {
                let obj_val = self.eval_expr(obj)?;
                match &obj_val {
                    Value::Tuple(vals) => {
                        // Field access on tuple: .0, .1, etc.
                        let idx: usize = field
                            .parse()
                            .map_err(|_| format!("Invalid tuple field: {}", field))?;
                        vals.get(idx)
                            .cloned()
                            .ok_or_else(|| format!("Tuple index {} out of bounds", idx))
                    }
                    _ => Err(format!("Cannot access field {} on {:?}", field, obj_val)),
                }
            }
            Expr::Range(start, end, inclusive, _span) => {
                let s = self.eval_expr(start)?;
                let e = self.eval_expr(end)?;
                match (&s, &e) {
                    (Value::Int(a), Value::Int(b)) => {
                        let end = if *inclusive { *b + 1 } else { *b };
                        let items: Vec<Value> = (*a..end).map(Value::Int).collect();
                        Ok(Value::List(items))
                    }
                    _ => Err(format!("Range requires integer bounds, got {:?}..{:?}", s, e)),
                }
            }
            Expr::Tuple(exprs, _span) => {
                let mut vals = Vec::new();
                for e in exprs {
                    vals.push(self.eval_expr(e)?);
                }
                Ok(Value::Tuple(vals))
            }
            Expr::List(exprs, _span) => {
                let mut vals = Vec::new();
                for e in exprs {
                    vals.push(self.eval_expr(e)?);
                }
                Ok(Value::List(vals))
            }
            Expr::Lambda(params, body, _span) => Ok(Value::Lambda {
                params: params.clone(),
                body: body.clone(),
                closure: self.env.clone(),
            }),
            Expr::StringInterp(parts, _span) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Lit(s) => result.push_str(s),
                        StringPart::Expr(e) => {
                            let val = self.eval_expr(e)?;
                            result.push_str(&format!("{}", val));
                        }
                    }
                }
                Ok(Value::Str(result))
            }
            Expr::If(cond, then_block, else_branch, _span) => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.is_truthy() {
                    self.eval_block(then_block)
                } else if let Some(else_expr) = else_branch {
                    self.eval_expr(else_expr)
                } else {
                    Ok(Value::Unit)
                }
            }
            Expr::Match(scrutinee, arms, _span) => {
                let val = self.eval_expr(scrutinee)?;
                for arm in arms {
                    let saved_env = self.env.clone();
                    if self.match_pattern(&arm.pattern, &val)? {
                        let result = self.eval_expr(&arm.body);
                        if result.is_err() {
                            self.env = saved_env;
                        }
                        return result;
                    }
                    self.env = saved_env;
                }
                Err(format!("No matching arm in match expression for value: {}", val))
            }
            Expr::Block(block, _span) => self.eval_block(block),
            Expr::Assign(target, value, _span) => {
                let val = self.eval_expr(value)?;
                match target.as_ref() {
                    Expr::Ident(name, _) => {
                        if !self.env.update(name, val.clone()) {
                            return Err(format!("Cannot assign to undefined variable: {}", name));
                        }
                        Ok(val)
                    }
                    _ => Err("Can only assign to identifiers".to_string()),
                }
            }
            Expr::CompoundAssign(target, op, value, _span) => {
                let current = self.eval_expr(target)?;
                let rhs = self.eval_expr(value)?;
                let result = self.eval_binop(op, &current, &rhs)?;
                match target.as_ref() {
                    Expr::Ident(name, _) => {
                        if !self.env.update(name, result.clone()) {
                            return Err(format!("Cannot assign to undefined variable: {}", name));
                        }
                        Ok(result)
                    }
                    _ => Err("Can only assign to identifiers".to_string()),
                }
            }
        }
    }

    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::Str(s) => Value::Str(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
        }
    }

    fn eval_binop(&self, op: &BinOp, left: &Value, right: &Value) -> Result<Value, String> {
        match (op, left, right) {
            // Integer arithmetic
            (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (BinOp::Mod, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Int(a % b))
                }
            }

            // Float arithmetic
            (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (BinOp::Mod, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),

            // Mixed int/float
            (BinOp::Add, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (BinOp::Add, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (BinOp::Sub, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (BinOp::Sub, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            (BinOp::Mul, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (BinOp::Mul, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            (BinOp::Div, Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            (BinOp::Div, Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),

            // String concatenation with +
            (BinOp::Add, Value::Str(a), Value::Str(b)) => {
                Ok(Value::Str(format!("{}{}", a, b)))
            }

            // Comparisons - integers
            (BinOp::Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Ne, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

            // Comparisons - strings
            (BinOp::Eq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Ne, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a > b)),

            // Comparisons - booleans
            (BinOp::Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Ne, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),

            // Comparisons - floats
            (BinOp::Eq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
            (BinOp::Ne, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a != b)),
            (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (BinOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

            // Logical operators
            (BinOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            (BinOp::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),

            // Concat (++)
            (BinOp::Concat, Value::List(a), Value::List(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::List(result))
            }
            (BinOp::Concat, Value::Str(a), Value::Str(b)) => {
                Ok(Value::Str(format!("{}{}", a, b)))
            }

            _ => Err(format!(
                "Unsupported binary operation {:?} on {} and {}",
                op, left, right
            )),
        }
    }

    fn eval_unop(&self, op: &UnaryOp, val: &Value) -> Result<Value, String> {
        match (op, val) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            _ => Err(format!("Unsupported unary operation {:?} on {}", op, val)),
        }
    }

    fn call_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, String> {
        match func {
            Value::BuiltinFn(name) => self.call_builtin(name, args),
            Value::Fn {
                params,
                body,
                closure,
                ..
            } => {
                let saved_env = self.env.clone();
                self.env = Env::child(closure);

                for (i, param) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Unit);
                    self.env.set(param.name.clone(), val);
                }

                // Also make sure all top-level functions are available
                for (name, val) in saved_env.bindings.iter() {
                    if matches!(val, Value::Fn { .. } | Value::BuiltinFn(_)) {
                        if self.env.get(name).is_none() {
                            self.env.set(name.clone(), val.clone());
                        }
                    }
                }

                let result = match self.eval_block_with_signals(body) {
                    Ok(v) => Ok(v),
                    Err(Ok(Signal::Return(v))) => Ok(v),
                    Err(Ok(Signal::Break)) => Err("break outside of loop".to_string()),
                    Err(Err(e)) => Err(e),
                };

                self.env = saved_env;
                result
            }
            Value::Lambda {
                params,
                body,
                closure,
            } => {
                let saved_env = self.env.clone();
                self.env = Env::child(closure);

                for (i, param) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Unit);
                    self.env.set(param.name.clone(), val);
                }

                // Make sure builtins and top-level functions are accessible
                for (name, val) in saved_env.bindings.iter() {
                    if matches!(val, Value::Fn { .. } | Value::BuiltinFn(_)) {
                        if self.env.get(name).is_none() {
                            self.env.set(name.clone(), val.clone());
                        }
                    }
                }

                let result = self.eval_expr(body);
                self.env = saved_env;
                result
            }
            _ => Err(format!("Cannot call non-function value: {}", func)),
        }
    }

    fn call_builtin(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match name {
            "println" => {
                if let Some(val) = args.first() {
                    println!("{}", val);
                } else {
                    println!();
                }
                Ok(Value::Unit)
            }
            "print" => {
                if let Some(val) = args.first() {
                    print!("{}", val);
                }
                Ok(Value::Unit)
            }
            "to_str" => {
                let val = args.first().ok_or("to_str requires 1 argument")?;
                Ok(Value::Str(format!("{}", val)))
            }
            "to_int" => {
                let val = args.first().ok_or("to_int requires 1 argument")?;
                match val {
                    Value::Str(s) => {
                        let n: i64 = s
                            .trim()
                            .parse()
                            .map_err(|e| format!("Cannot parse '{}' as int: {}", s, e))?;
                        Ok(Value::Int(n))
                    }
                    Value::Float(f) => Ok(Value::Int(*f as i64)),
                    Value::Int(n) => Ok(Value::Int(*n)),
                    _ => Err(format!("Cannot convert {} to int", val)),
                }
            }
            "to_float" => {
                let val = args.first().ok_or("to_float requires 1 argument")?;
                match val {
                    Value::Str(s) => {
                        let f: f64 = s
                            .trim()
                            .parse()
                            .map_err(|e| format!("Cannot parse '{}' as float: {}", s, e))?;
                        Ok(Value::Float(f))
                    }
                    Value::Int(n) => Ok(Value::Float(*n as f64)),
                    Value::Float(f) => Ok(Value::Float(*f)),
                    _ => Err(format!("Cannot convert {} to float", val)),
                }
            }
            "len" => {
                let val = args.first().ok_or("len requires 1 argument")?;
                match val {
                    Value::List(l) => Ok(Value::Int(l.len() as i64)),
                    Value::Str(s) => Ok(Value::Int(s.len() as i64)),
                    _ => Err(format!("Cannot get length of {}", val)),
                }
            }
            "push" => {
                let list = args.first().ok_or("push requires 2 arguments")?;
                let item = args.get(1).ok_or("push requires 2 arguments")?;
                match list {
                    Value::List(l) => {
                        let mut new_list = l.clone();
                        new_list.push(item.clone());
                        Ok(Value::List(new_list))
                    }
                    _ => Err(format!("push requires a list, got {}", list)),
                }
            }
            "map" => {
                let list = args.first().ok_or("map requires 2 arguments")?;
                let func = args.get(1).ok_or("map requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            result.push(self.call_function(func, &[item.clone()])?);
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err(format!("map requires a list, got {}", list)),
                }
            }
            "filter" => {
                let list = args.first().ok_or("filter requires 2 arguments")?;
                let func = args.get(1).ok_or("filter requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            let keep = self.call_function(func, &[item.clone()])?;
                            if keep.is_truthy() {
                                result.push(item.clone());
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err(format!("filter requires a list, got {}", list)),
                }
            }
            "fold" => {
                let list = args.first().ok_or("fold requires 3 arguments")?;
                let init = args.get(1).ok_or("fold requires 3 arguments")?;
                let func = args.get(2).ok_or("fold requires 3 arguments")?;
                match list {
                    Value::List(items) => {
                        let mut acc = init.clone();
                        for item in items {
                            acc = self.call_function(func, &[acc, item.clone()])?;
                        }
                        Ok(acc)
                    }
                    _ => Err(format!("fold requires a list, got {}", list)),
                }
            }
            "range" => {
                let start = args.first().ok_or("range requires 2 arguments")?;
                let end = args.get(1).ok_or("range requires 2 arguments")?;
                match (start, end) {
                    (Value::Int(a), Value::Int(b)) => {
                        let items: Vec<Value> = (*a..*b).map(Value::Int).collect();
                        Ok(Value::List(items))
                    }
                    _ => Err("range requires integer arguments".to_string()),
                }
            }
            "split" => {
                let s = args.first().ok_or("split requires 2 arguments")?;
                let delim = args.get(1).ok_or("split requires 2 arguments")?;
                match (s, delim) {
                    (Value::Str(s), Value::Str(d)) => {
                        let parts: Vec<Value> = s.split(d.as_str()).map(|p| Value::Str(p.to_string())).collect();
                        Ok(Value::List(parts))
                    }
                    _ => Err("split requires string arguments".to_string()),
                }
            }
            "join" => {
                let list = args.first().ok_or("join requires 2 arguments")?;
                let sep = args.get(1).ok_or("join requires 2 arguments")?;
                match (list, sep) {
                    (Value::List(items), Value::Str(s)) => {
                        let parts: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                        Ok(Value::Str(parts.join(s)))
                    }
                    _ => Err("join requires a list and string separator".to_string()),
                }
            }
            "trim" => {
                let s = args.first().ok_or("trim requires 1 argument")?;
                match s {
                    Value::Str(s) => Ok(Value::Str(s.trim().to_string())),
                    _ => Err("trim requires a string".to_string()),
                }
            }
            "contains" => {
                let container = args.first().ok_or("contains requires 2 arguments")?;
                let item = args.get(1).ok_or("contains requires 2 arguments")?;
                match container {
                    Value::Str(s) => match item {
                        Value::Str(sub) => Ok(Value::Bool(s.contains(sub.as_str()))),
                        _ => Err("contains on string requires string argument".to_string()),
                    },
                    Value::List(items) => {
                        let found = items.iter().any(|v| self.values_equal(v, item));
                        Ok(Value::Bool(found))
                    }
                    _ => Err("contains requires a string or list".to_string()),
                }
            }
            "sort" => {
                let list = args.first().ok_or("sort requires 1 argument")?;
                match list {
                    Value::List(items) => {
                        let mut sorted = items.clone();
                        sorted.sort_by(|a, b| self.compare_values(a, b));
                        Ok(Value::List(sorted))
                    }
                    _ => Err("sort requires a list".to_string()),
                }
            }
            "sort_by" => {
                let list = args.first().ok_or("sort_by requires 2 arguments")?;
                let func = args.get(1).ok_or("sort_by requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        let mut keyed: Vec<(Value, Value)> = Vec::new();
                        for item in items {
                            let key = self.call_function(func, &[item.clone()])?;
                            keyed.push((item.clone(), key));
                        }
                        keyed.sort_by(|a, b| self.compare_values(&a.1, &b.1));
                        let sorted: Vec<Value> = keyed.into_iter().map(|(v, _)| v).collect();
                        Ok(Value::List(sorted))
                    }
                    _ => Err("sort_by requires a list".to_string()),
                }
            }
            "rev" => {
                let list = args.first().ok_or("rev requires 1 argument")?;
                match list {
                    Value::List(items) => {
                        let mut reversed = items.clone();
                        reversed.reverse();
                        Ok(Value::List(reversed))
                    }
                    _ => Err("rev requires a list".to_string()),
                }
            }
            "enumerate" => {
                let list = args.first().ok_or("enumerate requires 1 argument")?;
                match list {
                    Value::List(items) => {
                        let result: Vec<Value> = items
                            .iter()
                            .enumerate()
                            .map(|(i, v)| Value::Tuple(vec![Value::Int(i as i64), v.clone()]))
                            .collect();
                        Ok(Value::List(result))
                    }
                    _ => Err("enumerate requires a list".to_string()),
                }
            }
            "zip" => {
                let l1 = args.first().ok_or("zip requires 2 arguments")?;
                let l2 = args.get(1).ok_or("zip requires 2 arguments")?;
                match (l1, l2) {
                    (Value::List(a), Value::List(b)) => {
                        let result: Vec<Value> = a
                            .iter()
                            .zip(b.iter())
                            .map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()]))
                            .collect();
                        Ok(Value::List(result))
                    }
                    _ => Err("zip requires two lists".to_string()),
                }
            }
            "flat_map" => {
                let list = args.first().ok_or("flat_map requires 2 arguments")?;
                let func = args.get(1).ok_or("flat_map requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            let mapped = self.call_function(func, &[item.clone()])?;
                            match mapped {
                                Value::List(inner) => result.extend(inner),
                                other => result.push(other),
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err("flat_map requires a list".to_string()),
                }
            }
            "any" => {
                let list = args.first().ok_or("any requires 2 arguments")?;
                let func = args.get(1).ok_or("any requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        for item in items {
                            let result = self.call_function(func, &[item.clone()])?;
                            if result.is_truthy() {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    _ => Err("any requires a list".to_string()),
                }
            }
            "all" => {
                let list = args.first().ok_or("all requires 2 arguments")?;
                let func = args.get(1).ok_or("all requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        for item in items {
                            let result = self.call_function(func, &[item.clone()])?;
                            if !result.is_truthy() {
                                return Ok(Value::Bool(false));
                            }
                        }
                        Ok(Value::Bool(true))
                    }
                    _ => Err("all requires a list".to_string()),
                }
            }
            "find" => {
                let list = args.first().ok_or("find requires 2 arguments")?;
                let func = args.get(1).ok_or("find requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        for item in items {
                            let result = self.call_function(func, &[item.clone()])?;
                            if result.is_truthy() {
                                return Ok(item.clone());
                            }
                        }
                        Ok(Value::Unit)
                    }
                    _ => Err("find requires a list".to_string()),
                }
            }
            "unique" => {
                let list = args.first().ok_or("unique requires 1 argument")?;
                match list {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            if !result.iter().any(|v| self.values_equal(v, item)) {
                                result.push(item.clone());
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err("unique requires a list".to_string()),
                }
            }
            "chunk" => {
                let list = args.first().ok_or("chunk requires 2 arguments")?;
                let size = args.get(1).ok_or("chunk requires 2 arguments")?;
                match (list, size) {
                    (Value::List(items), Value::Int(n)) => {
                        let n = *n as usize;
                        let result: Vec<Value> = items
                            .chunks(n)
                            .map(|c| Value::List(c.to_vec()))
                            .collect();
                        Ok(Value::List(result))
                    }
                    _ => Err("chunk requires a list and integer".to_string()),
                }
            }
            "take" => {
                let list = args.first().ok_or("take requires 2 arguments")?;
                let n = args.get(1).ok_or("take requires 2 arguments")?;
                match (list, n) {
                    (Value::List(items), Value::Int(n)) => {
                        let n = *n as usize;
                        Ok(Value::List(items[..n.min(items.len())].to_vec()))
                    }
                    _ => Err("take requires a list and integer".to_string()),
                }
            }
            "skip" => {
                let list = args.first().ok_or("skip requires 2 arguments")?;
                let n = args.get(1).ok_or("skip requires 2 arguments")?;
                match (list, n) {
                    (Value::List(items), Value::Int(n)) => {
                        let n = *n as usize;
                        Ok(Value::List(items[n.min(items.len())..].to_vec()))
                    }
                    _ => Err("skip requires a list and integer".to_string()),
                }
            }
            "min" => {
                let a = args.first().ok_or("min requires 2 arguments")?;
                let b = args.get(1).ok_or("min requires 2 arguments")?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.min(b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                    _ => Err("min requires comparable values".to_string()),
                }
            }
            "max" => {
                let a = args.first().ok_or("max requires 2 arguments")?;
                let b = args.get(1).ok_or("max requires 2 arguments")?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(*a.max(b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                    _ => Err("max requires comparable values".to_string()),
                }
            }
            "abs" => {
                let val = args.first().ok_or("abs requires 1 argument")?;
                match val {
                    Value::Int(n) => Ok(Value::Int(n.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err("abs requires a number".to_string()),
                }
            }
            "dbg" => {
                let val = args.first().ok_or("dbg requires 1 argument")?;
                eprintln!("[dbg] {:?}", val);
                Ok(val.clone())
            }
            "assert" => {
                let val = args.first().ok_or("assert requires 1 argument")?;
                if val.is_truthy() {
                    Ok(Value::Unit)
                } else {
                    Err("Assertion failed".to_string())
                }
            }
            "type_of" => {
                let val = args.first().ok_or("type_of requires 1 argument")?;
                let type_name = match val {
                    Value::Int(_) => "Int",
                    Value::Float(_) => "Float",
                    Value::Str(_) => "Str",
                    Value::Bool(_) => "Bool",
                    Value::Unit => "Unit",
                    Value::Tuple(_) => "Tuple",
                    Value::List(_) => "List",
                    Value::Fn { .. } => "Fn",
                    Value::Lambda { .. } => "Fn",
                    Value::BuiltinFn(_) => "Fn",
                };
                Ok(Value::Str(type_name.to_string()))
            }
            "read_file" => {
                let path = args.first().ok_or("read_file requires 1 argument")?;
                match path {
                    Value::Str(p) => match std::fs::read_to_string(p) {
                        Ok(content) => Ok(Value::Str(content)),
                        Err(e) => Err(format!("Cannot read file '{}': {}", p, e)),
                    },
                    _ => Err("read_file requires a string path".to_string()),
                }
            }
            "write_file" => {
                let path = args.first().ok_or("write_file requires 2 arguments")?;
                let content = args.get(1).ok_or("write_file requires 2 arguments")?;
                match (path, content) {
                    (Value::Str(p), Value::Str(c)) => match std::fs::write(p, c) {
                        Ok(_) => Ok(Value::Unit),
                        Err(e) => Err(format!("Cannot write file '{}': {}", p, e)),
                    },
                    _ => Err("write_file requires string arguments".to_string()),
                }
            }
            "read_line" => {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| format!("Cannot read line: {}", e))?;
                Ok(Value::Str(input.trim_end_matches('\n').to_string()))
            }
            "parse_json" => {
                // Basic JSON parsing - just return the string for now
                let val = args.first().ok_or("parse_json requires 1 argument")?;
                match val {
                    Value::Str(_) => Ok(val.clone()),
                    _ => Err("parse_json requires a string".to_string()),
                }
            }
            "group_by" => {
                let list = args.first().ok_or("group_by requires 2 arguments")?;
                let func = args.get(1).ok_or("group_by requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        let mut groups: Vec<(Value, Vec<Value>)> = Vec::new();
                        for item in items {
                            let key = self.call_function(func, &[item.clone()])?;
                            if let Some(group) = groups.iter_mut().find(|(k, _)| self.values_equal(k, &key)) {
                                group.1.push(item.clone());
                            } else {
                                groups.push((key, vec![item.clone()]));
                            }
                        }
                        let result: Vec<Value> = groups
                            .into_iter()
                            .map(|(k, v)| Value::Tuple(vec![k, Value::List(v)]))
                            .collect();
                        Ok(Value::List(result))
                    }
                    _ => Err("group_by requires a list".to_string()),
                }
            }
            "flatten" => {
                let list = args.first().ok_or("flatten requires 1 argument")?;
                match list {
                    Value::List(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            match item {
                                Value::List(inner) => result.extend(inner.clone()),
                                other => result.push(other.clone()),
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err("flatten requires a list".to_string()),
                }
            }
            "reduce" => {
                let list = args.first().ok_or("reduce requires 2 arguments")?;
                let func = args.get(1).ok_or("reduce requires 2 arguments")?;
                match list {
                    Value::List(items) => {
                        if items.is_empty() {
                            return Err("reduce on empty list".to_string());
                        }
                        let mut acc = items[0].clone();
                        for item in &items[1..] {
                            acc = self.call_function(func, &[acc, item.clone()])?;
                        }
                        Ok(acc)
                    }
                    _ => Err("reduce requires a list".to_string()),
                }
            }
            "replace" => {
                let s = args.first().ok_or("replace requires 3 arguments")?;
                let from = args.get(1).ok_or("replace requires 3 arguments")?;
                let to = args.get(2).ok_or("replace requires 3 arguments")?;
                match (s, from, to) {
                    (Value::Str(s), Value::Str(f), Value::Str(t)) => {
                        Ok(Value::Str(s.replace(f.as_str(), t)))
                    }
                    _ => Err("replace requires string arguments".to_string()),
                }
            }
            "starts_with" => {
                let s = args.first().ok_or("starts_with requires 2 arguments")?;
                let prefix = args.get(1).ok_or("starts_with requires 2 arguments")?;
                match (s, prefix) {
                    (Value::Str(s), Value::Str(p)) => Ok(Value::Bool(s.starts_with(p.as_str()))),
                    _ => Err("starts_with requires string arguments".to_string()),
                }
            }
            "ends_with" => {
                let s = args.first().ok_or("ends_with requires 2 arguments")?;
                let suffix = args.get(1).ok_or("ends_with requires 2 arguments")?;
                match (s, suffix) {
                    (Value::Str(s), Value::Str(p)) => Ok(Value::Bool(s.ends_with(p.as_str()))),
                    _ => Err("ends_with requires string arguments".to_string()),
                }
            }
            "to_upper" => {
                let s = args.first().ok_or("to_upper requires 1 argument")?;
                match s {
                    Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
                    _ => Err("to_upper requires a string".to_string()),
                }
            }
            "to_lower" => {
                let s = args.first().ok_or("to_lower requires 1 argument")?;
                match s {
                    Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
                    _ => Err("to_lower requires a string".to_string()),
                }
            }
            _ => Err(format!("Unknown builtin function: {}", name)),
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<(), String> {
        match pattern {
            Pattern::Ident(name) => {
                self.env.set(name.clone(), value.clone());
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            Pattern::Tuple(pats) => match value {
                Value::Tuple(vals) => {
                    if pats.len() != vals.len() {
                        return Err(format!(
                            "Tuple pattern has {} elements but value has {}",
                            pats.len(),
                            vals.len()
                        ));
                    }
                    for (p, v) in pats.iter().zip(vals.iter()) {
                        self.bind_pattern(p, v)?;
                    }
                    Ok(())
                }
                _ => Err(format!("Cannot destructure {} as tuple", value)),
            },
            Pattern::List(pats) => match value {
                Value::List(vals) => {
                    if pats.len() != vals.len() {
                        return Err(format!(
                            "List pattern has {} elements but value has {}",
                            pats.len(),
                            vals.len()
                        ));
                    }
                    for (p, v) in pats.iter().zip(vals.iter()) {
                        self.bind_pattern(p, v)?;
                    }
                    Ok(())
                }
                _ => Err(format!("Cannot destructure {} as list", value)),
            },
            Pattern::Literal(_) => {
                // Literal patterns in let bindings just check equality
                Ok(())
            }
            _ => Err(format!("Unsupported pattern in let binding: {:?}", pattern)),
        }
    }

    fn match_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<bool, String> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Ident(name) => {
                self.env.set(name.clone(), value.clone());
                Ok(true)
            }
            Pattern::Literal(lit) => {
                let pat_val = self.literal_to_value(lit);
                Ok(self.values_equal(&pat_val, value))
            }
            Pattern::Tuple(pats) => match value {
                Value::Tuple(vals) => {
                    if pats.len() != vals.len() {
                        return Ok(false);
                    }
                    for (p, v) in pats.iter().zip(vals.iter()) {
                        if !self.match_pattern(p, v)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },
            Pattern::List(pats) => match value {
                Value::List(vals) => {
                    if pats.len() != vals.len() {
                        return Ok(false);
                    }
                    for (p, v) in pats.iter().zip(vals.iter()) {
                        if !self.match_pattern(p, v)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            },
            Pattern::Or(pats) => {
                for p in pats {
                    if self.match_pattern(p, value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Pattern::Constructor(name, _pats) => {
                // For now, just match enum variants
                Err(format!("Constructor pattern matching not yet implemented for {}", name))
            }
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Tuple(a), Value::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            (Value::List(a), Value::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            _ => false,
        }
    }

    fn compare_values(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Str(a), Value::Str(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }

    fn value_to_iterable(&self, value: &Value) -> Result<Vec<Value>, String> {
        match value {
            Value::List(items) => Ok(items.clone()),
            Value::Str(s) => Ok(s.chars().map(|c| Value::Str(c.to_string())).collect()),
            _ => Err(format!("Cannot iterate over {}", value)),
        }
    }

    fn value_to_debug(&self, value: &Value) -> String {
        format!("{:?}", value)
    }
}
