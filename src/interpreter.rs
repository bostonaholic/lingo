//! Tree-walking evaluator for the Lingo programming language (Lisp dialect).
//!
//! Implements the evaluate/apply model with S-expression syntax.

use std::collections::HashMap;
use std::fmt;

use crate::ast::Expr;

// ---------------------------------------------------------------------------
// Section A: Value enum and Env
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
    List(Vec<Value>),
    Lambda {
        params: Vec<String>,
        body: Vec<Expr>,
        closure: Env,
    },
    Builtin {
        name: String,
        func: fn(&[Value]) -> Result<Value, String>,
    },
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
            Value::Nil => write!(f, "nil"),
            Value::List(vals) => {
                write!(f, "(")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Lambda { .. } => write!(f, "<lambda>"),
            Value::Builtin { name, .. } => write!(f, "<builtin:{}>", name),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "Int({})", n),
            Value::Float(n) => write!(f, "Float({})", n),
            Value::Str(s) => write!(f, "Str({:?})", s),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Nil => write!(f, "Nil"),
            Value::List(vals) => write!(f, "List({:?})", vals),
            Value::Lambda { params, .. } => write!(f, "Lambda({:?})", params),
            Value::Builtin { name, .. } => write!(f, "Builtin({})", name),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Builtin { name: a, .. }, Value::Builtin { name: b, .. }) => a == b,
            _ => false,
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Bool(false) | Value::Int(0) | Value::Nil)
}

#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Box<Env>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Env {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn child(&self) -> Self {
        Env {
            bindings: HashMap::new(),
            parent: Some(Box::new(self.clone())),
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

    pub fn update(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.update(name, value)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }

    pub fn binding_names(&self) -> Vec<String> {
        self.bindings.keys().cloned().collect()
    }
}

// ---------------------------------------------------------------------------
// Section B: Core evaluator
// ---------------------------------------------------------------------------

pub fn evaluate(expr: &Expr, env: &mut Env) -> Result<Value, String> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(f) => Ok(Value::Float(*f)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Nil => Ok(Value::Nil),
        Expr::Symbol(name) => env
            .get(name)
            .ok_or_else(|| format!("Undefined variable: {}", name)),
        Expr::List(elems) => {
            if elems.is_empty() {
                return Ok(Value::Nil);
            }
            // Check for special forms
            if let Expr::Symbol(head) = &elems[0] {
                match head.as_str() {
                    "def" => return eval_define(&elems[1..], env),
                    "defn" => return eval_defn(&elems[1..], env),
                    "fn" => return eval_fn(&elems[1..], env),
                    "if" => return eval_if(&elems[1..], env),
                    "cond" => return eval_cond(&elems[1..], env),
                    "let" => return eval_let(&elems[1..], env),
                    "do" => return eval_begin(&elems[1..], env),
                    "quote" => return eval_quote(&elems[1..]),
                    "set" => return eval_set(&elems[1..], env),
                    "and" => return eval_and(&elems[1..], env),
                    "or" => return eval_or(&elems[1..], env),
                    "while" => return eval_while(&elems[1..], env),
                    "match" => return eval_match(&elems[1..], env),
                    "->" => return eval_thread(&elems[1..], env),
                    _ => {}
                }
            }
            // General function application: evaluate all, apply head to rest
            let vals: Vec<Value> = elems
                .iter()
                .map(|e| evaluate(e, env))
                .collect::<Result<Vec<_>, _>>()?;
            let (func, args) = vals.split_first().unwrap();
            apply_function(func, args)
        }
    }
}

pub fn apply_function(func: &Value, args: &[Value]) -> Result<Value, String> {
    call_value(func, args)
}

/// Shared helper that both `apply_function` and higher-order builtins use.
pub fn call_value(func: &Value, args: &[Value]) -> Result<Value, String> {
    match func {
        Value::Lambda {
            params,
            body,
            closure,
        } => {
            if args.len() != params.len() {
                return Err(format!(
                    "Expected {} arguments, got {}",
                    params.len(),
                    args.len()
                ));
            }
            let mut local = closure.child();
            for (param, arg) in params.iter().zip(args.iter()) {
                local.set(param.clone(), arg.clone());
            }
            let mut result = Value::Nil;
            for expr in body {
                result = evaluate(expr, &mut local)?;
            }
            Ok(result)
        }
        Value::Builtin { func, .. } => func(args),
        other => Err(format!("Not a function: {}", other)),
    }
}

// ---------------------------------------------------------------------------
// Section C: Special form handlers
// ---------------------------------------------------------------------------

/// `(def name expr)`
fn eval_define(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("def requires exactly 2 arguments".to_string());
    }
    let name = args[0].as_symbol()?.to_string();
    let value = evaluate(&args[1], env)?;
    env.set(name, value);
    Ok(Value::Nil)
}

/// `(defn name (params) body...)`
fn eval_defn(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("defn requires name, params, and body".to_string());
    }
    let name = args[0].as_symbol()?.to_string();
    let param_exprs = args[1].as_list()?;
    let params: Vec<String> = param_exprs
        .iter()
        .map(|p| p.as_symbol().map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let body: Vec<Expr> = args[2..].to_vec();
    let lambda = Value::Lambda {
        params,
        body,
        closure: env.clone(),
    };
    env.set(name, lambda);
    Ok(Value::Nil)
}

/// `(fn (params) body...)`
fn eval_fn(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("fn requires params and body".to_string());
    }
    let param_exprs = args[0].as_list()?;
    let params: Vec<String> = param_exprs
        .iter()
        .map(|p| p.as_symbol().map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let body: Vec<Expr> = args[1..].to_vec();
    Ok(Value::Lambda {
        params,
        body,
        closure: env.clone(),
    })
}

/// `(if test then [else])`
fn eval_if(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err("if requires 2 or 3 arguments".to_string());
    }
    let test = evaluate(&args[0], env)?;
    if is_truthy(&test) {
        evaluate(&args[1], env)
    } else if args.len() == 3 {
        evaluate(&args[2], env)
    } else {
        Ok(Value::Nil)
    }
}

/// `(cond (test1 expr1) ... (else exprN))`
fn eval_cond(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    for clause in args {
        let elems = clause.as_list()?;
        if elems.len() != 2 {
            return Err("cond clause must have exactly 2 elements".to_string());
        }
        // Check for `else` keyword
        if let Expr::Symbol(s) = &elems[0]
            && s == "else"
        {
            return evaluate(&elems[1], env);
        }
        let test = evaluate(&elems[0], env)?;
        if is_truthy(&test) {
            return evaluate(&elems[1], env);
        }
    }
    Ok(Value::Nil)
}

/// `(let ((x 1) (y 2)) body...)`
fn eval_let(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("let requires bindings and body".to_string());
    }
    let bindings = args[0].as_list()?;
    let mut local = env.child();
    for binding in bindings {
        let pair = binding.as_list()?;
        if pair.len() != 2 {
            return Err("let binding must have exactly 2 elements".to_string());
        }
        let name = pair[0].as_symbol()?.to_string();
        let value = evaluate(&pair[1], &mut local)?;
        local.set(name, value);
    }
    let mut result = Value::Nil;
    for expr in &args[1..] {
        result = evaluate(expr, &mut local)?;
    }
    Ok(result)
}

/// `(do e1 e2 ... en)`
fn eval_begin(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let mut result = Value::Nil;
    for expr in args {
        result = evaluate(expr, env)?;
    }
    Ok(result)
}

/// `(quote expr)` -- return expr as data
fn eval_quote(args: &[Expr]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("quote requires exactly 1 argument".to_string());
    }
    Ok(expr_to_value(&args[0]))
}

/// Convert an Expr tree to a Value tree for `quote`.
fn expr_to_value(expr: &Expr) -> Value {
    match expr {
        Expr::Int(n) => Value::Int(*n),
        Expr::Float(f) => Value::Float(*f),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Nil => Value::Nil,
        Expr::Symbol(s) => Value::Str(s.clone()),
        Expr::List(elems) => Value::List(elems.iter().map(expr_to_value).collect()),
    }
}

/// `(set name expr)` -- mutate existing binding
fn eval_set(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("set requires exactly 2 arguments".to_string());
    }
    let name = args[0].as_symbol()?;
    let value = evaluate(&args[1], env)?;
    env.update(name, value)?;
    Ok(Value::Nil)
}

/// `(and a b c)` -- short-circuit, return last truthy or first falsy
fn eval_and(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let mut result = Value::Bool(true);
    for expr in args {
        result = evaluate(expr, env)?;
        if !is_truthy(&result) {
            return Ok(result);
        }
    }
    Ok(result)
}

/// `(or a b c)` -- short-circuit, return first truthy or last falsy
fn eval_or(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    let mut result = Value::Bool(false);
    for expr in args {
        result = evaluate(expr, env)?;
        if is_truthy(&result) {
            return Ok(result);
        }
    }
    Ok(result)
}

/// `(while test body...)` -- loop while test is truthy, return Nil
fn eval_while(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.is_empty() {
        return Err("while requires a test and body".to_string());
    }
    let test = &args[0];
    let body = &args[1..];
    loop {
        let cond = evaluate(test, env)?;
        if !is_truthy(&cond) {
            break;
        }
        for expr in body {
            evaluate(expr, env)?;
        }
    }
    Ok(Value::Nil)
}

/// `(match expr (pattern1 result1) ... (_ default))`
fn eval_match(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("match requires scrutinee and at least one arm".to_string());
    }
    let scrutinee = evaluate(&args[0], env)?;
    for arm in &args[1..] {
        let elems = arm.as_list()?;
        if elems.len() != 2 {
            return Err("match arm must have exactly 2 elements".to_string());
        }
        let pattern = &elems[0];
        let result = &elems[1];
        if match_pattern(pattern, &scrutinee, env)? {
            return evaluate(result, env);
        }
    }
    Err(format!("No matching pattern for value: {}", scrutinee))
}

/// Check if a pattern matches a value. Supports literal and wildcard patterns.
fn match_pattern(pattern: &Expr, value: &Value, _env: &mut Env) -> Result<bool, String> {
    match pattern {
        Expr::Symbol(s) if s == "_" => Ok(true),
        Expr::Int(n) => Ok(matches!(value, Value::Int(v) if v == n)),
        Expr::Float(f) => Ok(matches!(value, Value::Float(v) if v == f)),
        Expr::Str(s) => Ok(matches!(value, Value::Str(v) if v == s)),
        Expr::Bool(b) => Ok(matches!(value, Value::Bool(v) if v == b)),
        Expr::Nil => Ok(matches!(value, Value::Nil)),
        _ => Err(format!("Unsupported match pattern: {}", pattern)),
    }
}

/// `(-> val (f a b) (g c))` desugars to `(g (f val a b) c)`
fn eval_thread(args: &[Expr], env: &mut Env) -> Result<Value, String> {
    if args.is_empty() {
        return Err("-> requires at least one argument".to_string());
    }
    let mut result = evaluate(&args[0], env)?;
    for step in &args[1..] {
        match step {
            Expr::List(elems) => {
                if elems.is_empty() {
                    return Err("-> step cannot be an empty list".to_string());
                }
                // Evaluate the function
                let func = evaluate(&elems[0], env)?;
                // Build args: result is first arg, then the rest
                let mut call_args = vec![result];
                for arg_expr in &elems[1..] {
                    call_args.push(evaluate(arg_expr, env)?);
                }
                result = call_value(&func, &call_args)?;
            }
            Expr::Symbol(_) => {
                // Bare symbol: call as unary function
                let func = evaluate(step, env)?;
                result = call_value(&func, &[result])?;
            }
            _ => return Err(format!("-> step must be a list or symbol, got: {}", step)),
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Section D: Builtin functions
// ---------------------------------------------------------------------------

type BuiltinFn = fn(&[Value]) -> Result<Value, String>;

fn register_builtins(env: &mut Env) {
    let builtins: Vec<(&str, BuiltinFn)> = vec![
        // Arithmetic
        ("+", builtin_add),
        ("-", builtin_sub),
        ("*", builtin_mul),
        ("/", builtin_div),
        ("mod", builtin_mod),
        ("abs", builtin_abs),
        ("min", builtin_min),
        ("max", builtin_max),
        // Comparison
        ("=", builtin_eq),
        ("<", builtin_lt),
        (">", builtin_gt),
        ("<=", builtin_le),
        (">=", builtin_ge),
        // Logic
        ("not", builtin_not),
        // List
        ("list", builtin_list),
        ("cons", builtin_cons),
        ("first", builtin_first),
        ("rest", builtin_rest),
        ("nth", builtin_nth),
        ("len", builtin_length),
        ("cat", builtin_append),
        ("rev", builtin_reverse),
        ("map", builtin_map),
        ("filter", builtin_filter),
        ("fold", builtin_fold),
        ("each", builtin_for_each),
        ("flat", builtin_flatten),
        ("zip", builtin_zip),
        ("take", builtin_take),
        ("drop", builtin_drop),
        ("sort", builtin_sort),
        ("sortby", builtin_sort_by),
        ("any", builtin_any),
        ("all", builtin_all),
        ("find", builtin_find),
        ("uniq", builtin_unique),
        ("chunk", builtin_chunk),
        ("enumerate", builtin_enumerate),
        ("groupby", builtin_group_by),
        ("range", builtin_range),
        // String
        ("str", builtin_str),
        ("strlen", builtin_string_length),
        ("substring", builtin_substring),
        ("split", builtin_split),
        ("join", builtin_join),
        ("trim", builtin_trim),
        ("has", builtin_contains),
        ("replace", builtin_replace),
        ("starts-with", builtin_starts_with),
        ("ends-with", builtin_ends_with),
        ("upcase", builtin_upper_case),
        ("downcase", builtin_lower_case),
        // Type
        ("type-of", builtin_type_of),
        ("int?", builtin_is_int),
        ("float?", builtin_is_float),
        ("string?", builtin_is_string),
        ("bool?", builtin_is_bool),
        ("list?", builtin_is_list),
        ("nil?", builtin_is_nil),
        ("number?", builtin_is_number),
        // Conversion
        ("->int", builtin_to_int),
        ("->float", builtin_to_float),
        ("->str", builtin_to_str),
        // I/O
        ("println", builtin_println),
        ("print", builtin_print),
        ("readline", builtin_read_line),
        ("readfile", builtin_read_file),
        ("writefile", builtin_write_file),
        // Debug/Test
        ("dbg", builtin_dbg),
        ("assert", builtin_assert),
        ("assert-eq", builtin_assert_eq),
        ("assert-ne", builtin_assert_ne),
        ("assert-true", builtin_assert_true),
        ("assert-false", builtin_assert_false),
    ];

    for (name, func) in builtins {
        env.set(
            name.to_string(),
            Value::Builtin {
                name: name.to_string(),
                func,
            },
        );
    }
}

// -- Arithmetic builtins --

fn builtin_add(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = numeric_op(&result, arg, "+", |a, b| a + b, |a, b| a + b)?;
    }
    Ok(result)
}

fn builtin_sub(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("- requires at least 1 argument".to_string());
    }
    if args.len() == 1 {
        // Unary negate
        return match &args[0] {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            other => Err(format!("Cannot negate: {}", other)),
        };
    }
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = numeric_op(&result, arg, "-", |a, b| a - b, |a, b| a - b)?;
    }
    Ok(result)
}

fn builtin_mul(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Int(1));
    }
    let mut result = args[0].clone();
    for arg in &args[1..] {
        result = numeric_op(&result, arg, "*", |a, b| a * b, |a, b| a * b)?;
    }
    Ok(result)
}

fn builtin_div(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("/ requires exactly 2 arguments".to_string());
    }
    // Check for division by zero
    match &args[1] {
        Value::Int(0) => return Err("Division by zero".to_string()),
        Value::Float(f) if *f == 0.0 => return Err("Division by zero".to_string()),
        _ => {}
    }
    numeric_op(&args[0], &args[1], "/", |a, b| a / b, |a, b| a / b)
}

fn builtin_mod(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("% requires exactly 2 arguments".to_string());
    }
    if let Value::Int(0) = &args[1] {
        return Err("Modulo by zero".to_string());
    }
    numeric_op(&args[0], &args[1], "mod", |a, b| a % b, |a, b| a % b)
}

fn builtin_abs(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("abs requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        other => Err(format!("abs: expected number, got {}", other)),
    }
}

fn builtin_min(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("min requires exactly 2 arguments".to_string());
    }
    let ord = compare_values(&args[0], &args[1])?;
    if ord == std::cmp::Ordering::Less || ord == std::cmp::Ordering::Equal {
        Ok(args[0].clone())
    } else {
        Ok(args[1].clone())
    }
}

fn builtin_max(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("max requires exactly 2 arguments".to_string());
    }
    let ord = compare_values(&args[0], &args[1])?;
    if ord == std::cmp::Ordering::Greater || ord == std::cmp::Ordering::Equal {
        Ok(args[0].clone())
    } else {
        Ok(args[1].clone())
    }
}

fn numeric_op(
    a: &Value,
    b: &Value,
    op_name: &str,
    int_op: fn(i64, i64) -> i64,
    float_op: fn(f64, f64) -> f64,
) -> Result<Value, String> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(int_op(*x, *y))),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(float_op(*x, *y))),
        (Value::Int(x), Value::Float(y)) => Ok(Value::Float(float_op(*x as f64, *y))),
        (Value::Float(x), Value::Int(y)) => Ok(Value::Float(float_op(*x, *y as f64))),
        _ => Err(format!("{}: expected numbers, got {} and {}", op_name, a, b)),
    }
}

fn compare_values(a: &Value, b: &Value) -> Result<std::cmp::Ordering, String> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        (Value::Float(x), Value::Float(y)) => x
            .partial_cmp(y)
            .ok_or_else(|| "Cannot compare NaN".to_string()),
        (Value::Int(x), Value::Float(y)) => (*x as f64)
            .partial_cmp(y)
            .ok_or_else(|| "Cannot compare NaN".to_string()),
        (Value::Float(x), Value::Int(y)) => x
            .partial_cmp(&(*y as f64))
            .ok_or_else(|| "Cannot compare NaN".to_string()),
        (Value::Str(x), Value::Str(y)) => Ok(x.cmp(y)),
        _ => Err(format!("Cannot compare {} and {}", a, b)),
    }
}

// -- Comparison builtins --

fn builtin_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("= requires exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(args[0] == args[1]))
}

fn builtin_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("< requires exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        compare_values(&args[0], &args[1])? == std::cmp::Ordering::Less,
    ))
}

fn builtin_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("> requires exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        compare_values(&args[0], &args[1])? == std::cmp::Ordering::Greater,
    ))
}

fn builtin_le(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("<= requires exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        compare_values(&args[0], &args[1])? != std::cmp::Ordering::Greater,
    ))
}

fn builtin_ge(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(">= requires exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(
        compare_values(&args[0], &args[1])? != std::cmp::Ordering::Less,
    ))
}

// -- Logic builtins --

fn builtin_not(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("not requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(!is_truthy(&args[0])))
}

// -- List builtins --

fn builtin_list(args: &[Value]) -> Result<Value, String> {
    Ok(Value::List(args.to_vec()))
}

fn builtin_cons(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("cons requires exactly 2 arguments".to_string());
    }
    match &args[1] {
        Value::List(list) => {
            let mut new = vec![args[0].clone()];
            new.extend(list.iter().cloned());
            Ok(Value::List(new))
        }
        _ => Err(format!("cons: second argument must be a list, got {}", args[1])),
    }
}

fn builtin_first(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("first requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            if list.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(list[0].clone())
            }
        }
        _ => Err(format!("first: expected list, got {}", args[0])),
    }
}

fn builtin_rest(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("rest requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            if list.is_empty() {
                Ok(Value::List(vec![]))
            } else {
                Ok(Value::List(list[1..].to_vec()))
            }
        }
        _ => Err(format!("rest: expected list, got {}", args[0])),
    }
}

fn builtin_nth(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("nth requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::List(list), Value::Int(i)) => {
            let idx = *i as usize;
            if idx < list.len() {
                Ok(list[idx].clone())
            } else {
                Err(format!("nth: index {} out of bounds for list of length {}", i, list.len()))
            }
        }
        _ => Err(format!("nth: expected (list, int), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_length(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("len requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => Ok(Value::Int(list.len() as i64)),
        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
        _ => Err(format!("len: expected list or string, got {}", args[0])),
    }
}

fn builtin_append(args: &[Value]) -> Result<Value, String> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::List(list) => result.extend(list.iter().cloned()),
            _ => return Err(format!("cat: expected list, got {}", arg)),
        }
    }
    Ok(Value::List(result))
}

fn builtin_reverse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("rev requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            let mut rev = list.clone();
            rev.reverse();
            Ok(Value::List(rev))
        }
        _ => Err(format!("rev: expected list, got {}", args[0])),
    }
}

fn builtin_map(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("map requires exactly 2 arguments (list, function)".to_string());
    }
    // Accept either (list, fn) or (fn, list) for flexibility
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("map: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    let mut result = Vec::new();
    for item in list {
        result.push(call_value(func, std::slice::from_ref(item))?);
    }
    Ok(Value::List(result))
}

fn builtin_filter(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("filter requires exactly 2 arguments (list, predicate)".to_string());
    }
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("filter: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    let mut result = Vec::new();
    for item in list {
        let val = call_value(func, std::slice::from_ref(item))?;
        if is_truthy(&val) {
            result.push(item.clone());
        }
    }
    Ok(Value::List(result))
}

fn builtin_fold(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("fold requires exactly 3 arguments".to_string());
    }
    // Accept (list, initial, function) or (initial, function, list)
    let (acc_init, func, list) = match (&args[0], &args[1], &args[2]) {
        (Value::List(l), init, f @ (Value::Lambda { .. } | Value::Builtin { .. })) => {
            (init.clone(), f, l.clone())
        }
        (init, f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => {
            (init.clone(), f, l.clone())
        }
        _ => return Err("fold: expected a list, initial value, and function".to_string()),
    };
    let mut acc = acc_init;
    for item in &list {
        acc = call_value(func, &[acc, item.clone()])?;
    }
    Ok(acc)
}

fn builtin_for_each(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("each requires exactly 2 arguments (function, list)".to_string());
    }
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("each: expected a function and a list, got {} and {}", args[0], args[1])),
    };
    for item in list {
        call_value(func, std::slice::from_ref(item))?;
    }
    Ok(Value::Nil)
}

fn builtin_flatten(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("flat requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            let mut result = Vec::new();
            for item in list {
                match item {
                    Value::List(inner) => result.extend(inner.iter().cloned()),
                    other => result.push(other.clone()),
                }
            }
            Ok(Value::List(result))
        }
        _ => Err(format!("flat: expected list, got {}", args[0])),
    }
}

fn builtin_zip(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("zip requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::List(a), Value::List(b)) => {
            let result: Vec<Value> = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| Value::List(vec![x.clone(), y.clone()]))
                .collect();
            Ok(Value::List(result))
        }
        _ => Err(format!("zip: expected two lists, got {} and {}", args[0], args[1])),
    }
}

fn builtin_take(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("take requires exactly 2 arguments (n, list)".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Int(n), Value::List(list)) => {
            let n = (*n).max(0) as usize;
            Ok(Value::List(list.iter().take(n).cloned().collect()))
        }
        _ => Err(format!("take: expected (int, list), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_drop(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("drop requires exactly 2 arguments (n, list)".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Int(n), Value::List(list)) => {
            let n = (*n).max(0) as usize;
            Ok(Value::List(list.iter().skip(n).cloned().collect()))
        }
        _ => Err(format!("drop: expected (int, list), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_sort(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("sort requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            let mut sorted = list.clone();
            let mut err = None;
            sorted.sort_by(|a, b| match compare_values(a, b) {
                Ok(ord) => ord,
                Err(e) => {
                    err = Some(e);
                    std::cmp::Ordering::Equal
                }
            });
            if let Some(e) = err {
                return Err(e);
            }
            Ok(Value::List(sorted))
        }
        _ => Err(format!("sort: expected list, got {}", args[0])),
    }
}

fn builtin_sort_by(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("sortby requires exactly 2 arguments (list, function)".to_string());
    }
    let (func, list_ref) = match (&args[0], &args[1]) {
        (Value::List(_), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, &args[0]),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(_)) => (f, &args[1]),
        _ => return Err(format!("sortby: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    let list = match list_ref {
        Value::List(l) => l,
        _ => unreachable!(),
    };
    // Map each element through the function, then sort by the result
    let mut indexed: Vec<(Value, Value)> = list
        .iter()
        .map(|item| {
            let key = call_value(func, std::slice::from_ref(item))?;
            Ok((item.clone(), key))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut err = None;
    indexed.sort_by(|(_, ka), (_, kb)| match compare_values(ka, kb) {
        Ok(ord) => ord,
        Err(e) => {
            err = Some(e);
            std::cmp::Ordering::Equal
        }
    });
    if let Some(e) = err {
        return Err(e);
    }
    Ok(Value::List(indexed.into_iter().map(|(v, _)| v).collect()))
}

fn builtin_any(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("any requires exactly 2 arguments (list, predicate)".to_string());
    }
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("any: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    for item in list {
        let val = call_value(func, std::slice::from_ref(item))?;
        if is_truthy(&val) {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}

fn builtin_all(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("all requires exactly 2 arguments (list, predicate)".to_string());
    }
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("all: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    for item in list {
        let val = call_value(func, std::slice::from_ref(item))?;
        if !is_truthy(&val) {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn builtin_find(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("find requires exactly 2 arguments (list, predicate)".to_string());
    }
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("find: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    for item in list {
        let val = call_value(func, std::slice::from_ref(item))?;
        if is_truthy(&val) {
            return Ok(item.clone());
        }
    }
    Ok(Value::Nil)
}

fn builtin_unique(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("uniq requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            let mut seen = Vec::new();
            let mut result = Vec::new();
            for item in list {
                let s = format!("{:?}", item);
                if !seen.contains(&s) {
                    seen.push(s);
                    result.push(item.clone());
                }
            }
            Ok(Value::List(result))
        }
        _ => Err(format!("uniq: expected list, got {}", args[0])),
    }
}

fn builtin_chunk(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("chunk requires exactly 2 arguments (size, list)".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Int(n), Value::List(list)) => {
            if *n <= 0 {
                return Err("chunk: size must be positive".to_string());
            }
            let n = *n as usize;
            let result: Vec<Value> = list.chunks(n).map(|c| Value::List(c.to_vec())).collect();
            Ok(Value::List(result))
        }
        _ => Err(format!("chunk: expected (int, list), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_enumerate(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("enumerate requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(list) => {
            let result: Vec<Value> = list
                .iter()
                .enumerate()
                .map(|(i, v)| Value::List(vec![Value::Int(i as i64), v.clone()]))
                .collect();
            Ok(Value::List(result))
        }
        _ => Err(format!("enum: expected list, got {}", args[0])),
    }
}

fn builtin_group_by(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("groupby requires exactly 2 arguments (list, function)".to_string());
    }
    let (func, list) = match (&args[0], &args[1]) {
        (Value::List(l), f @ (Value::Lambda { .. } | Value::Builtin { .. })) => (f, l),
        (f @ (Value::Lambda { .. } | Value::Builtin { .. }), Value::List(l)) => (f, l),
        _ => return Err(format!("groupby: expected a list and a function, got {} and {}", args[0], args[1])),
    };
    // Group into a list of (key, [values]) pairs preserving insertion order
    let mut keys: Vec<String> = Vec::new();
    let mut groups: HashMap<String, (Value, Vec<Value>)> = HashMap::new();
    for item in list {
        let key = call_value(func, std::slice::from_ref(item))?;
        let key_str = format!("{}", key);
        groups
            .entry(key_str.clone())
            .or_insert_with(|| {
                keys.push(key_str.clone());
                (key.clone(), Vec::new())
            })
            .1
            .push(item.clone());
    }
    let result: Vec<Value> = keys
        .iter()
        .map(|k| {
            let (key, vals) = groups.remove(k).unwrap();
            Value::List(vec![key, Value::List(vals)])
        })
        .collect();
    Ok(Value::List(result))
}

fn builtin_range(args: &[Value]) -> Result<Value, String> {
    match args.len() {
        1 => match &args[0] {
            Value::Int(end) => {
                let list: Vec<Value> = (0..*end).map(Value::Int).collect();
                Ok(Value::List(list))
            }
            _ => Err(format!("range: expected int, got {}", args[0])),
        },
        2 => match (&args[0], &args[1]) {
            (Value::Int(start), Value::Int(end)) => {
                let list: Vec<Value> = (*start..*end).map(Value::Int).collect();
                Ok(Value::List(list))
            }
            _ => Err(format!(
                "range: expected (int, int), got ({}, {})",
                args[0], args[1]
            )),
        },
        3 => match (&args[0], &args[1], &args[2]) {
            (Value::Int(start), Value::Int(end), Value::Int(step)) => {
                if *step == 0 {
                    return Err("range: step cannot be zero".to_string());
                }
                let mut list = Vec::new();
                let mut i = *start;
                if *step > 0 {
                    while i < *end {
                        list.push(Value::Int(i));
                        i += step;
                    }
                } else {
                    while i > *end {
                        list.push(Value::Int(i));
                        i += step;
                    }
                }
                Ok(Value::List(list))
            }
            _ => Err(format!(
                "range: expected (int, int, int), got ({}, {}, {})",
                args[0], args[1], args[2]
            )),
        },
        _ => Err("range requires 1, 2, or 3 arguments".to_string()),
    }
}

// -- String builtins --

fn builtin_str(args: &[Value]) -> Result<Value, String> {
    let mut result = String::new();
    for arg in args {
        result.push_str(&format!("{}", arg));
    }
    Ok(Value::Str(result))
}

fn builtin_string_length(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("strlen requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
        _ => Err(format!("strlen: expected string, got {}", args[0])),
    }
}

fn builtin_substring(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("substring requires exactly 3 arguments (string, start, end)".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::Str(s), Value::Int(start), Value::Int(end)) => {
            let start = (*start).max(0) as usize;
            let end = (*end).min(s.len() as i64).max(0) as usize;
            if start > end || start > s.len() {
                Ok(Value::Str(String::new()))
            } else {
                Ok(Value::Str(s[start..end].to_string()))
            }
        }
        _ => Err(format!(
            "substring: expected (string, int, int), got ({}, {}, {})",
            args[0], args[1], args[2]
        )),
    }
}

fn builtin_split(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("split requires exactly 2 arguments (string, delimiter)".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Str(delim)) => {
            let parts: Vec<Value> = s.split(delim.as_str()).map(|p| Value::Str(p.to_string())).collect();
            Ok(Value::List(parts))
        }
        _ => Err(format!("split: expected (string, string), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_join(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("join requires exactly 2 arguments (separator, list)".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Str(sep), Value::List(list)) => {
            let parts: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
            Ok(Value::Str(parts.join(sep)))
        }
        _ => Err(format!("join: expected (string, list), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_trim(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("trim requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Str(s.trim().to_string())),
        _ => Err(format!("trim: expected string, got {}", args[0])),
    }
}

fn builtin_contains(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("has requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Str(haystack), Value::Str(needle)) => Ok(Value::Bool(haystack.contains(needle.as_str()))),
        (Value::List(list), val) => Ok(Value::Bool(list.contains(val))),
        _ => Err(format!("has: expected (string, string) or (list, value), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_replace(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("replace requires exactly 3 arguments (string, from, to)".to_string());
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::Str(s), Value::Str(from), Value::Str(to)) => {
            Ok(Value::Str(s.replace(from.as_str(), to)))
        }
        _ => Err(format!(
            "replace: expected (string, string, string), got ({}, {}, {})",
            args[0], args[1], args[2]
        )),
    }
}

fn builtin_starts_with(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("starts-with requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Str(prefix)) => Ok(Value::Bool(s.starts_with(prefix.as_str()))),
        _ => Err(format!("starts-with: expected (string, string), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_ends_with(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("ends-with requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Str(s), Value::Str(suffix)) => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
        _ => Err(format!("ends-with: expected (string, string), got ({}, {})", args[0], args[1])),
    }
}

fn builtin_upper_case(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("upcase requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
        _ => Err(format!("upcase: expected string, got {}", args[0])),
    }
}

fn builtin_lower_case(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("downcase requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
        _ => Err(format!("downcase: expected string, got {}", args[0])),
    }
}

// -- Type builtins --

fn builtin_type_of(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("type-of requires exactly 1 argument".to_string());
    }
    let t = match &args[0] {
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Str(_) => "string",
        Value::Bool(_) => "bool",
        Value::Nil => "nil",
        Value::List(_) => "list",
        Value::Lambda { .. } => "lambda",
        Value::Builtin { .. } => "builtin",
    };
    Ok(Value::Str(t.to_string()))
}

fn builtin_is_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("int? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Int(_))))
}

fn builtin_is_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("float? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Float(_))))
}

fn builtin_is_string(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("string? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Str(_))))
}

fn builtin_is_bool(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("bool? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

fn builtin_is_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("list? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}

fn builtin_is_nil(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("nil? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

fn builtin_is_number(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("number? requires exactly 1 argument".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Int(_) | Value::Float(_))))
}

// -- Conversion builtins --

fn builtin_to_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("->int requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::Str(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("->int: cannot convert '{}' to int", s)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        other => Err(format!("->int: cannot convert {} to int", other)),
    }
}

fn builtin_to_float(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("->float requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Str(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("->float: cannot convert '{}' to float", s)),
        other => Err(format!("->float: cannot convert {} to float", other)),
    }
}

fn builtin_to_str(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("->str requires exactly 1 argument".to_string());
    }
    Ok(Value::Str(format!("{}", args[0])))
}

// -- I/O builtins --

fn builtin_println(args: &[Value]) -> Result<Value, String> {
    let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Nil)
}

fn builtin_print(args: &[Value]) -> Result<Value, String> {
    let parts: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    print!("{}", parts.join(" "));
    Ok(Value::Nil)
}

fn builtin_read_line(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("readline takes no arguments".to_string());
    }
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    Ok(Value::Str(input.trim_end_matches('\n').to_string()))
}

fn builtin_read_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("readfile requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::Str(path) => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("readfile: {}", e))?;
            Ok(Value::Str(content))
        }
        _ => Err(format!("readfile: expected string, got {}", args[0])),
    }
}

fn builtin_write_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("writefile requires exactly 2 arguments (path, content)".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Str(path), Value::Str(content)) => {
            std::fs::write(path, content).map_err(|e| format!("writefile: {}", e))?;
            Ok(Value::Nil)
        }
        _ => Err(format!(
            "writefile: expected (string, string), got ({}, {})",
            args[0], args[1]
        )),
    }
}

// -- Debug/Test builtins --

fn builtin_dbg(args: &[Value]) -> Result<Value, String> {
    for arg in args {
        eprintln!("[dbg] {:?}", arg);
    }
    if args.len() == 1 {
        Ok(args[0].clone())
    } else {
        Ok(Value::Nil)
    }
}

fn builtin_assert(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("assert requires exactly 1 argument".to_string());
    }
    if is_truthy(&args[0]) {
        Ok(Value::Nil)
    } else {
        Err(format!("[assert] assertion failed: {}", args[0]))
    }
}

fn builtin_assert_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("assert-eq requires exactly 2 arguments".to_string());
    }
    if args[0] == args[1] {
        Ok(Value::Nil)
    } else {
        Err(format!(
            "[assert] expected: {}, got: {}",
            args[0], args[1]
        ))
    }
}

fn builtin_assert_ne(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("assert-ne requires exactly 2 arguments".to_string());
    }
    if args[0] != args[1] {
        Ok(Value::Nil)
    } else {
        Err(format!(
            "[assert] expected values to differ, both are: {}",
            args[0]
        ))
    }
}

fn builtin_assert_true(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("assert-true requires exactly 1 argument".to_string());
    }
    if is_truthy(&args[0]) {
        Ok(Value::Nil)
    } else {
        Err(format!(
            "[assert] expected truthy, got: {}",
            args[0]
        ))
    }
}

fn builtin_assert_false(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("assert-false requires exactly 1 argument".to_string());
    }
    if !is_truthy(&args[0]) {
        Ok(Value::Nil)
    } else {
        Err(format!(
            "[assert] expected falsy, got: {}",
            args[0]
        ))
    }
}

// ---------------------------------------------------------------------------
// Interpreter struct -- public API
// ---------------------------------------------------------------------------

pub struct Interpreter {
    pub env: Env,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Env::new();
        register_builtins(&mut env);
        Interpreter { env }
    }

    /// Lex, parse, evaluate all top-level forms. Call `main` if defined.
    pub fn run(&mut self, source: &str) -> Result<(), String> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let exprs = parser.parse()?;

        for expr in &exprs {
            evaluate(expr, &mut self.env)?;
        }

        // Call main if defined
        if let Some(main_fn) = self.env.get("main") {
            call_value(&main_fn, &[])?;
        }

        Ok(())
    }

    /// Lex, parse, evaluate all forms, return results.
    pub fn run_source(&mut self, source: &str) -> Result<Vec<Value>, String> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let exprs = parser.parse()?;

        let mut results = Vec::new();
        for expr in &exprs {
            results.push(evaluate(expr, &mut self.env)?);
        }
        Ok(results)
    }

    /// Lex, parse, evaluate, return last result (used by REPL and tests).
    pub fn eval_source(&mut self, source: &str) -> Result<Value, String> {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let exprs = parser.parse()?;

        let mut result = Value::Nil;
        for expr in &exprs {
            result = evaluate(expr, &mut self.env)?;
        }
        Ok(result)
    }

    /// Lex, parse, evaluate all forms but do not call main (used by test runner).
    pub fn load_source(&mut self, source: &str) {
        let mut lexer = crate::lexer::Lexer::new(source);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => return,
        };
        let mut parser = crate::parser::Parser::new(tokens);
        let exprs = match parser.parse() {
            Ok(e) => e,
            Err(_) => return,
        };
        for expr in &exprs {
            let _ = evaluate(expr, &mut self.env);
        }
    }

    /// Call a function value with arguments (used by test runner).
    pub fn call_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, String> {
        call_value(func, args)
    }
}
