use std::collections::HashMap;

use crate::ast::*;
use crate::error::LangError;

const MAX_CALL_DEPTH: usize = 64;

/// Runtime value for the VX interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4(Box<[f32; 16]>),
    Color([f32; 4]),
    Text2d(Box<TextData>),
    Text3d(Box<TextData>),
    Null,
    Fn {
        params: Vec<Param>,
        body: Spanned<Expr>,
        closure: Snapshot,
    },
    NativeFn {
        name: String,
        arity: usize,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Str(v) => write!(f, "{v}"),
            Value::Vec2([x, y]) => write!(f, "vec2({x}, {y})"),
            Value::Vec3([x, y, z]) => write!(f, "vec3({x}, {y}, {z})"),
            Value::Vec4([x, y, z, w]) => write!(f, "vec4({x}, {y}, {z}, {w})"),
            Value::Color([r, g, b, a]) => write!(f, "color({r}, {g}, {b}, {a})"),
            Value::Mat4(_) => write!(f, "mat4(...)"),
            Value::Text2d(d) => write!(f, "text2d({:?})", d.content),
            Value::Text3d(d) => write!(f, "text3d({:?})", d.content),
            Value::Null => write!(f, "null"),
            Value::Fn { .. } => write!(f, "<fn>"),
            Value::NativeFn { name, .. } => write!(f, "<builtin:{name}>"),
        }
    }
}

/// Flat snapshot of bindings captured by a closure.
pub type Snapshot = HashMap<String, Value>;


/// Lexical-scope environment with a call-depth counter.
pub struct Env {
    frames: Vec<HashMap<String, Value>>,
    depth: usize,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Self { frames: vec![HashMap::new()], depth: 0 }
    }

    pub fn from_snapshot(snap: &Snapshot) -> Self {
        let mut env = Self::new();
        for (k, v) in snap {
            env.set(k, v.clone());
        }
        env
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut out = HashMap::new();
        for frame in &self.frames {
            out.extend(frame.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        out
    }

    pub fn push_frame(&mut self) -> Result<(), LangError> {
        if self.depth >= MAX_CALL_DEPTH {
            return Err(LangError::ParseError {
                msg: "maximum call depth exceeded".into(),
                offset: 0,
            });
        }
        self.frames.push(HashMap::new());
        self.depth += 1;
        Ok(())
    }

    pub fn pop_frame(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
            self.depth -= 1;
        }
    }

    pub fn set(&mut self, name: &str, val: Value) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), val);
        }
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), LangError> {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), val);
                return Ok(());
            }
        }
        Err(LangError::ParseError {
            msg: format!("undefined variable '{name}'"),
            offset: 0,
        })
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        None
    }
}

// ── helpers ────────────────────────────────────────────────────────────────

fn err(msg: impl Into<String>) -> LangError {
    LangError::ParseError { msg: msg.into(), offset: 0 }
}

fn coerce_float(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

// ── expression evaluator ───────────────────────────────────────────────────

pub fn eval_expr(expr: &Spanned<Expr>, env: &mut Env) -> Result<Value, LangError> {
    match &expr.node {
        Expr::Lit(lit) => Ok(eval_lit(lit)),
        Expr::Ident(name) => env.get(name).cloned().ok_or_else(|| err(format!("undefined variable '{name}'"))),
        Expr::Unary { op, expr: inner } => eval_unary(op, inner, env),
        Expr::Binary { op, lhs, rhs } => eval_binary(op, lhs, rhs, env),
        Expr::Block(stmts) => eval_block(stmts, env),
        Expr::If { cond, then_block, else_block } => match eval_expr(cond, env)? {
            Value::Bool(true) => eval_expr(then_block, env),
            Value::Bool(false) => else_block.as_ref().map(|eb| eval_expr(eb, env)).unwrap_or(Ok(Value::Null)),
            _ => Err(err("if condition must be bool")),
        },
        Expr::Call { callee, args } => {
            let fn_val = eval_expr(callee, env)?;
            let arg_vals: Result<Vec<_>, _> = args.iter().map(|a| eval_expr(a, env)).collect();
            call_fn(fn_val, arg_vals?)
        }
        Expr::Field { obj, name } => {
            let val = eval_expr(obj, env)?;
            eval_field(&val, name, &expr.span)
        }
        Expr::Index { .. } => Err(err("index operator not yet implemented")),
        Expr::Text2d(props) => {
            let data = eval_text_props(props, env)?;
            Ok(Value::Text2d(Box::new(data)))
        }
        Expr::Text3d(props) => {
            let data = eval_text_props(props, env)?;
            Ok(Value::Text3d(Box::new(data)))
        }
    }
}

fn eval_lit(lit: &Lit) -> Value {
    match lit {
        Lit::Int(v) => Value::Int(*v),
        Lit::Float(v) => Value::Float(*v),
        Lit::Bool(v) => Value::Bool(*v),
        Lit::Str(v) => Value::Str(v.clone()),
        Lit::Null => Value::Null,
    }
}

fn eval_unary(op: &UnOp, inner: &Spanned<Expr>, env: &mut Env) -> Result<Value, LangError> {
    let v = eval_expr(inner, env)?;
    match (op, &v) {
        (UnOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
        (UnOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
        (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        _ => Err(err(format!("cannot apply {op:?} to {v}"))),
    }
}

fn eval_binary(op: &BinOp, lhs: &Spanned<Expr>, rhs: &Spanned<Expr>, env: &mut Env) -> Result<Value, LangError> {
    let l = eval_expr(lhs, env)?;
    let r = eval_expr(rhs, env)?;
    match op {
        BinOp::Add => match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
            _ => Err(err(format!("cannot add {l} and {r}"))),
        },
        BinOp::Sub => numeric_op(&l, &r, i64::wrapping_sub, |a, b| a - b),
        BinOp::Mul => numeric_op(&l, &r, i64::wrapping_mul, |a, b| a * b),
        BinOp::Div => {
            if matches!((&l, &r), (Value::Int(_), Value::Int(0))) { return Err(err("division by zero")); }
            numeric_op(&l, &r, i64::wrapping_div, |a, b| a / b)
        }
        BinOp::Rem => numeric_op(&l, &r, i64::wrapping_rem, |a, b| a % b),
        BinOp::Pow => match (coerce_float(&l), coerce_float(&r)) {
            (Some(a), Some(b)) => Ok(Value::Float(a.powf(b))),
            _ => Err(err(format!("cannot raise {l} to {r}"))),
        },
        BinOp::Eq => Ok(Value::Bool(l == r)),
        BinOp::NotEq => Ok(Value::Bool(l != r)),
        BinOp::Lt => cmp_op(&l, &r, |o| o.is_lt()),
        BinOp::LtEq => cmp_op(&l, &r, |o| o.is_le()),
        BinOp::Gt => cmp_op(&l, &r, |o| o.is_gt()),
        BinOp::GtEq => cmp_op(&l, &r, |o| o.is_ge()),
        BinOp::And => match (&l, &r) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(err("&& requires bool operands")),
        },
        BinOp::Or => match (&l, &r) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(err("|| requires bool operands")),
        },
        BinOp::Range => Err(err("range not yet implemented as a value")),
    }
}

fn numeric_op(l: &Value, r: &Value, int_op: impl Fn(i64, i64) -> i64, float_op: impl Fn(f64, f64) -> f64) -> Result<Value, LangError> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
        _ => Err(err(format!("numeric operator not applicable to {l} and {r}"))),
    }
}

fn cmp_op(l: &Value, r: &Value, pred: impl Fn(std::cmp::Ordering) -> bool) -> Result<Value, LangError> {
    let ord = match (l, r) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).ok_or_else(|| err("NaN in comparison"))?,
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).ok_or_else(|| err("NaN"))?,
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).ok_or_else(|| err("NaN"))?,
        (Value::Str(a), Value::Str(b)) => a.cmp(b),
        _ => return Err(err(format!("cannot compare {l} and {r}"))),
    };
    Ok(Value::Bool(pred(ord)))
}

fn eval_field(val: &Value, name: &str, span: &std::ops::Range<usize>) -> Result<Value, LangError> {
    match (val, name) {
        (Value::Vec2([x, _]), "x") => Ok(Value::Float(*x as f64)),
        (Value::Vec2([_, y]), "y") => Ok(Value::Float(*y as f64)),
        (Value::Vec3([x, _, _]), "x") => Ok(Value::Float(*x as f64)),
        (Value::Vec3([_, y, _]), "y") => Ok(Value::Float(*y as f64)),
        (Value::Vec3([_, _, z]), "z") => Ok(Value::Float(*z as f64)),
        (Value::Vec4([x, _, _, _]), "x") => Ok(Value::Float(*x as f64)),
        (Value::Vec4([_, y, _, _]), "y") => Ok(Value::Float(*y as f64)),
        (Value::Vec4([_, _, z, _]), "z") => Ok(Value::Float(*z as f64)),
        (Value::Vec4([_, _, _, w]), "w") => Ok(Value::Float(*w as f64)),
        (Value::Color([r, _, _, _]), "r") => Ok(Value::Float(*r as f64)),
        (Value::Color([_, g, _, _]), "g") => Ok(Value::Float(*g as f64)),
        (Value::Color([_, _, b, _]), "b") => Ok(Value::Float(*b as f64)),
        (Value::Color([_, _, _, a]), "a") => Ok(Value::Float(*a as f64)),
        _ => Err(LangError::ParseError { msg: format!("no field '{name}' on {val}"), offset: span.start }),
    }
}

// ── statement evaluator ────────────────────────────────────────────────────

fn eval_stmt(stmt: &Spanned<Stmt>, env: &mut Env) -> Result<Option<Value>, LangError> {
    match &stmt.node {
        Stmt::Let { name, init, .. } => {
            let val = init.as_ref().map(|e| eval_expr(e, env)).transpose()?.unwrap_or(Value::Null);
            env.set(name, val);
            Ok(None)
        }
        Stmt::Assign { target, op, value } => {
            let rval = eval_expr(value, env)?;
            if let Expr::Ident(name) = &target.node {
                let new_val = match op {
                    AssignOp::Assign => rval,
                    AssignOp::AddAssign => {
                        let cur = env.get(name).cloned().ok_or_else(|| err(format!("undefined '{name}'")))?;
                        match (cur, rval) {
                            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                            (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
                            _ => return Err(err("+= type mismatch")),
                        }
                    }
                    AssignOp::SubAssign => {
                        let cur = env.get(name).cloned().ok_or_else(|| err(format!("undefined '{name}'")))?;
                        match (cur, rval) {
                            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                            _ => return Err(err("-= type mismatch")),
                        }
                    }
                    AssignOp::MulAssign => {
                        let cur = env.get(name).cloned().ok_or_else(|| err(format!("undefined '{name}'")))?;
                        match (cur, rval) {
                            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                            _ => return Err(err("*= type mismatch")),
                        }
                    }
                    AssignOp::DivAssign => {
                        let cur = env.get(name).cloned().ok_or_else(|| err(format!("undefined '{name}'")))?;
                        match (cur, rval) {
                            (Value::Int(a), Value::Int(b)) if b != 0 => Value::Int(a / b),
                            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                            _ => return Err(err("/= type mismatch or division by zero")),
                        }
                    }
                };
                env.assign(name, new_val)?;
            } else {
                return Err(err("complex assignment targets not yet supported"));
            }
            Ok(None)
        }
        Stmt::Return(val) => {
            let v = val.as_ref().map(|e| eval_expr(e, env)).transpose()?.unwrap_or(Value::Null);
            Ok(Some(v))
        }
        Stmt::For { var, iter, body } => {
            let iter_val = eval_expr(iter, env)?;
            match iter_val {
                Value::Int(n) => {
                    for i in 0..n {
                        env.push_frame()?;
                        env.set(var, Value::Int(i));
                        let result = eval_expr(body, env);
                        env.pop_frame();
                        result?;
                    }
                }
                _ => return Err(err("for loop requires integer value as upper bound")),
            }
            Ok(None)
        }
        Stmt::Expr(expr) => { eval_expr(expr, env)?; Ok(None) }
    }
}

fn eval_block(stmts: &[Spanned<Stmt>], env: &mut Env) -> Result<Value, LangError> {
    env.push_frame()?;
    let mut last = Value::Null;
    for stmt in stmts {
        match eval_stmt(stmt, env)? {
            Some(ret) => { env.pop_frame(); return Ok(ret); }
            None => {
                if let Stmt::Expr(e) = &stmt.node {
                    last = eval_expr(e, env)?;
                }
            }
        }
    }
    env.pop_frame();
    Ok(last)
}

// ── function calls ─────────────────────────────────────────────────────────

fn call_fn(fn_val: Value, args: Vec<Value>) -> Result<Value, LangError> {
    match fn_val {
        Value::Fn { params, body, closure } => {
            if args.len() != params.len() {
                return Err(err(format!("function expects {} arg(s), got {}", params.len(), args.len())));
            }
            let mut call_env = Env::from_snapshot(&closure);
            call_env.push_frame()?;
            for (param, val) in params.iter().zip(args) { call_env.set(&param.name, val); }
            let result = eval_expr(&body, &mut call_env)?;
            call_env.pop_frame();
            Ok(result)
        }
        Value::NativeFn { name, arity } => {
            if args.len() != arity {
                return Err(err(format!("built-in '{name}' expects {arity} arg(s), got {}", args.len())));
            }
            dispatch_native(&name, args)
        }
        _ => Err(err("value is not callable")),
    }
}

// ── module evaluator ───────────────────────────────────────────────────────

pub fn eval_module(module: &Module, env: &mut Env) -> Result<(), LangError> {
    for decl in &module.decls {
        match &decl.node {
            Decl::Let { name, init, .. } => {
                let val = eval_expr(init, env)?;
                env.set(name, val);
            }
            Decl::Fn { name, params, body, .. } => {
                let fn_val = Value::Fn { params: params.clone(), body: body.clone(), closure: env.snapshot() };
                env.set(name, fn_val);
            }
        }
    }
    Ok(())
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn run(src: &str) -> Env {
        let module = Parser::new(src).unwrap().parse_module().unwrap();
        let mut env = Env::new();
        eval_module(&module, &mut env).unwrap();
        env
    }

    fn get(env: &Env, name: &str) -> Value { env.get(name).cloned().unwrap() }

    #[test] fn let_int() { assert_eq!(get(&run("let x: int = 7;"), "x"), Value::Int(7)); }
    #[test] fn let_float() { assert_eq!(get(&run("let pi: float = 3.14;"), "pi"), Value::Float(3.14)); }
    #[test] fn let_bool() { assert_eq!(get(&run("let t: bool = true;"), "t"), Value::Bool(true)); }
    #[test] fn let_null() { assert_eq!(get(&run("let n: bool = null;"), "n"), Value::Null); }
    #[test] fn arithmetic() { assert_eq!(get(&run("let r: int = 2 + 3 * 4;"), "r"), Value::Int(14)); }
    #[test] fn comparison() { assert_eq!(get(&run("let ok: bool = 3 < 5;"), "ok"), Value::Bool(true)); }
    #[test] fn string_concat() { assert_eq!(get(&run(r#"let s: str = "foo" + "bar";"#), "s"), Value::Str("foobar".into())); }
    #[test] fn fn_defined() {
        let env = run("fn double(n: int) -> int { n; } let r: int = double(21);");
        assert_eq!(get(&env, "r"), Value::Int(21));
    }
    #[test] fn negation() { assert_eq!(get(&run("let n: int = -5;"), "n"), Value::Int(-5)); }
    #[test] fn not_bool() { assert_eq!(get(&run("let b: bool = !true;"), "b"), Value::Bool(false)); }
    #[test] fn if_true() { assert_eq!(get(&run("let v: int = if true { 1; } else { 2; };"), "v"), Value::Int(1)); }
    #[test] fn if_false() { assert_eq!(get(&run("let v: int = if false { 1; } else { 2; };"), "v"), Value::Int(2)); }
}

// ── text geometry ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TextAlign { Left, #[default] Center, Right }

/// Evaluated runtime representation of a text geometry object.
#[derive(Debug, Clone, PartialEq)]
pub struct TextData {
    pub content: String,
    pub font: String,
    pub size: f64,
    pub color: [f32; 4],
    pub align: TextAlign,
    pub position: [f32; 3],
    pub italic: bool,
    pub bold: bool,
    pub tracking: f64,
    pub line_height: f64,
    pub wrap_width: f64,
    // text3d only
    pub depth: f64,
    pub bevel_depth: f64,
    pub bevel_resolution: i64,
    pub rotation: [f32; 3],
}

impl Default for TextData {
    fn default() -> Self {
        Self {
            content: String::new(),
            font: "Inter".into(),
            size: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            position: [0.0, 0.0, 0.0],
            italic: false,
            bold: false,
            tracking: 1.0,
            line_height: 1.0,
            wrap_width: 0.0,
            depth: 0.1,
            bevel_depth: 0.0,
            bevel_resolution: 4,
            rotation: [0.0, 0.0, 0.0],
        }
    }
}

pub fn eval_text_props(props: &crate::ast::TextProps, env: &mut Env) -> Result<TextData, LangError> {
    let mut data = TextData::default();
    for (key, expr) in &props.fields {
        let val = eval_expr(expr, env)?;
        match key.as_str() {
            "content" => match val {
                Value::Str(s) => data.content = s,
                _ => return Err(err("text 'content' must be str")),
            },
            "font" => match val {
                Value::Str(s) => data.font = s,
                _ => return Err(err("text 'font' must be str")),
            },
            "size" => match val {
                Value::Float(f) => data.size = f,
                Value::Int(i) => data.size = i as f64,
                _ => return Err(err("text 'size' must be float")),
            },
            "color" => match val {
                Value::Color([r, g, b, a]) => data.color = [r, g, b, a],
                Value::Vec4([r, g, b, a]) => data.color = [r, g, b, a],
                _ => return Err(err("text 'color' must be color or vec4")),
            },
            "align" => match val {
                Value::Str(s) => data.align = match s.as_str() {
                    "left" => TextAlign::Left,
                    "center" => TextAlign::Center,
                    "right" => TextAlign::Right,
                    _ => return Err(err(format!("unknown align '{s}'; use left|center|right"))),
                },
                _ => return Err(err("text 'align' must be str")),
            },
            "position" => match val {
                Value::Vec3([x, y, z]) => data.position = [x, y, z],
                _ => return Err(err("text 'position' must be vec3")),
            },
            "rotation" => match val {
                Value::Vec3([x, y, z]) => data.rotation = [x, y, z],
                _ => return Err(err("text 'rotation' must be vec3")),
            },
            "italic" => match val {
                Value::Bool(b) => data.italic = b,
                _ => return Err(err("text 'italic' must be bool")),
            },
            "bold" => match val {
                Value::Bool(b) => data.bold = b,
                _ => return Err(err("text 'bold' must be bool")),
            },
            "tracking" => match val {
                Value::Float(f) => data.tracking = f,
                Value::Int(i) => data.tracking = i as f64,
                _ => return Err(err("text 'tracking' must be float")),
            },
            "line_height" => match val {
                Value::Float(f) => data.line_height = f,
                Value::Int(i) => data.line_height = i as f64,
                _ => return Err(err("text 'line_height' must be float")),
            },
            "wrap_width" => match val {
                Value::Float(f) => data.wrap_width = f,
                Value::Int(i) => data.wrap_width = i as f64,
                _ => return Err(err("text 'wrap_width' must be float")),
            },
            "depth" => match val {
                Value::Float(f) => data.depth = f,
                Value::Int(i) => data.depth = i as f64,
                _ => return Err(err("text 'depth' must be float")),
            },
            "bevel_depth" => match val {
                Value::Float(f) => data.bevel_depth = f,
                Value::Int(i) => data.bevel_depth = i as f64,
                _ => return Err(err("text 'bevel_depth' must be float")),
            },
            "bevel_resolution" => match val {
                Value::Int(i) => data.bevel_resolution = i,
                _ => return Err(err("text 'bevel_resolution' must be int")),
            },
            unknown => return Err(err(format!("unknown text property '{unknown}'"))),
        }
    }
    if data.content.is_empty() {
        return Err(err("text 'content' is required"));
    }
    Ok(data)
}

// ── built-in functions ─────────────────────────────────────────────────────

fn to_f64(v: &Value, name: &str) -> Result<f64, LangError> {
    match v {
        Value::Float(f) => Ok(*f),
        Value::Int(i) => Ok(*i as f64),
        _ => Err(err(format!("'{name}' expects a numeric argument"))),
    }
}

fn dispatch_native(name: &str, args: Vec<Value>) -> Result<Value, LangError> {
    match name {
        // ── 1-arg math ──────────────────────────────────────────────────
        "abs" => {
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(err("abs: expected numeric")),
            }
        }
        "sqrt"  => Ok(Value::Float(to_f64(&args[0], "sqrt")?.sqrt())),
        "floor" => Ok(Value::Float(to_f64(&args[0], "floor")?.floor())),
        "ceil"  => Ok(Value::Float(to_f64(&args[0], "ceil")?.ceil())),
        "round" => Ok(Value::Float(to_f64(&args[0], "round")?.round())),
        "sin"   => Ok(Value::Float(to_f64(&args[0], "sin")?.sin())),
        "cos"   => Ok(Value::Float(to_f64(&args[0], "cos")?.cos())),
        "tan"   => Ok(Value::Float(to_f64(&args[0], "tan")?.tan())),
        "log"   => Ok(Value::Float(to_f64(&args[0], "log")?.ln())),
        "exp"   => Ok(Value::Float(to_f64(&args[0], "exp")?.exp())),
        // ── 2-arg math ──────────────────────────────────────────────────
        "pow"   => Ok(Value::Float(to_f64(&args[0], "pow")?.powf(to_f64(&args[1], "pow")?))),
        "atan2" => Ok(Value::Float(to_f64(&args[0], "atan2")?.atan2(to_f64(&args[1], "atan2")?))),
        "min"   => {
            let a = to_f64(&args[0], "min")?;
            let b = to_f64(&args[1], "min")?;
            Ok(Value::Float(a.min(b)))
        }
        "max"   => {
            let a = to_f64(&args[0], "max")?;
            let b = to_f64(&args[1], "max")?;
            Ok(Value::Float(a.max(b)))
        }
        // ── 3-arg math ──────────────────────────────────────────────────
        "clamp" => {
            let v  = to_f64(&args[0], "clamp")?;
            let lo = to_f64(&args[1], "clamp")?;
            let hi = to_f64(&args[2], "clamp")?;
            Ok(Value::Float(v.clamp(lo, hi)))
        }
        "lerp" => {
            let a = to_f64(&args[0], "lerp")?;
            let b = to_f64(&args[1], "lerp")?;
            let t = to_f64(&args[2], "lerp")?;
            Ok(Value::Float(a + (b - a) * t))
        }
        // ── string functions ─────────────────────────────────────────────
        "len" => match &args[0] {
            Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
            _ => Err(err("len: expected str")),
        },
        "upper" => match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
            _ => Err(err("upper: expected str")),
        },
        "lower" => match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
            _ => Err(err("lower: expected str")),
        },
        "trim" => match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.trim().to_string())),
            _ => Err(err("trim: expected str")),
        },
        "to_str" => Ok(Value::Str(format!("{}", args[0]))),
        "to_int" => match &args[0] {
            Value::Str(s) => s.trim().parse::<i64>()
                .map(Value::Int)
                .map_err(|_| err(format!("to_int: cannot parse '{s}' as int"))),
            Value::Float(f) => Ok(Value::Int(*f as i64)),
            Value::Int(i) => Ok(Value::Int(*i)),
            _ => Err(err("to_int: expected str or numeric")),
        },
        "to_float" => match &args[0] {
            Value::Str(s) => s.trim().parse::<f64>()
                .map(Value::Float)
                .map_err(|_| err(format!("to_float: cannot parse '{s}' as float"))),
            Value::Int(i) => Ok(Value::Float(*i as f64)),
            Value::Float(f) => Ok(Value::Float(*f)),
            _ => Err(err("to_float: expected str or numeric")),
        },
        unknown => Err(err(format!("unknown built-in '{unknown}'"))),
    }
}

// ── environment bootstrap ─────────────────────────────────────────────────

/// Inject all built-in constants and functions into `env` before user code runs.
pub fn register_builtins(env: &mut Env) {
    // numeric constants
    env.set("PI",  Value::Float(std::f64::consts::PI));
    env.set("TAU", Value::Float(std::f64::consts::TAU));
    env.set("E",   Value::Float(std::f64::consts::E));
    env.set("INF", Value::Float(f64::INFINITY));

    // 1-arg math
    for name in ["abs", "sqrt", "floor", "ceil", "round", "sin", "cos", "tan", "log", "exp"] {
        env.set(name, Value::NativeFn { name: name.to_string(), arity: 1 });
    }
    // 2-arg math
    for name in ["pow", "atan2", "min", "max"] {
        env.set(name, Value::NativeFn { name: name.to_string(), arity: 2 });
    }
    // 3-arg math
    for name in ["clamp", "lerp"] {
        env.set(name, Value::NativeFn { name: name.to_string(), arity: 3 });
    }
    // string functions
    for name in ["len", "upper", "lower", "trim", "to_str", "to_int", "to_float"] {
        let arity = 1;
        env.set(name, Value::NativeFn { name: name.to_string(), arity });
    }
}

// ── built-in tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod builtin_tests {
    use super::*;
    use crate::parser::Parser;

    fn run(src: &str) -> Env {
        let module = Parser::new(src).unwrap().parse_module().unwrap();
        let mut env = Env::new();
        register_builtins(&mut env);
        eval_module(&module, &mut env).unwrap();
        env
    }

    fn get(env: &Env, name: &str) -> Value { env.get(name).cloned().unwrap() }
    fn f(v: Value) -> f64 { match v { Value::Float(f) => f, _ => panic!("not float") } }

    #[test] fn pi_constant() { let v = f(get(&run("let x: float = PI;"), "x")); assert!((v - std::f64::consts::PI).abs() < 1e-10); }
    #[test] fn tau_constant() { let v = f(get(&run("let x: float = TAU;"), "x")); assert!((v - std::f64::consts::TAU).abs() < 1e-10); }
    #[test] fn e_constant()  { let v = f(get(&run("let x: float = E;"), "x")); assert!((v - std::f64::consts::E).abs() < 1e-10); }
    #[test] fn sqrt_4()  { let v = f(get(&run("let x: float = sqrt(4.0);"), "x")); assert!((v - 2.0).abs() < 1e-10); }
    #[test] fn abs_neg()  { let v = f(get(&run("let x: float = abs(-3.0);"), "x")); assert!((v - 3.0).abs() < 1e-10); }
    #[test] fn floor_val() { let v = f(get(&run("let x: float = floor(3.9);"), "x")); assert_eq!(v, 3.0); }
    #[test] fn ceil_val()  { let v = f(get(&run("let x: float = ceil(3.1);"), "x")); assert_eq!(v, 4.0); }
    #[test] fn clamp_val() { let v = f(get(&run("let x: float = clamp(5.0, 0.0, 3.0);"), "x")); assert_eq!(v, 3.0); }
    #[test] fn lerp_half() { let v = f(get(&run("let x: float = lerp(0.0, 10.0, 0.5);"), "x")); assert!((v - 5.0).abs() < 1e-10); }
    #[test] fn min_val()   { let v = f(get(&run("let x: float = min(3.0, 7.0);"), "x")); assert_eq!(v, 3.0); }
    #[test] fn max_val()   { let v = f(get(&run("let x: float = max(3.0, 7.0);"), "x")); assert_eq!(v, 7.0); }
    #[test] fn pow_val()   { let v = f(get(&run("let x: float = pow(2.0, 3.0);"), "x")); assert!((v - 8.0).abs() < 1e-10); }
    #[test] fn sin_zero()  { let v = f(get(&run("let x: float = sin(0.0);"), "x")); assert!(v.abs() < 1e-10); }
    #[test] fn cos_zero()  { let v = f(get(&run("let x: float = cos(0.0);"), "x")); assert!((v - 1.0).abs() < 1e-10); }
    #[test] fn len_str()   { assert_eq!(get(&run(r#"let x: int = len("hello");"#), "x"), Value::Int(5)); }
    #[test] fn upper_str() { assert_eq!(get(&run(r#"let x: str = upper("hi");"#), "x"), Value::Str("HI".into())); }
    #[test] fn lower_str() { assert_eq!(get(&run(r#"let x: str = lower("HI");"#), "x"), Value::Str("hi".into())); }
    #[test] fn trim_str()  { assert_eq!(get(&run(r#"let x: str = trim("  hi  ");"#), "x"), Value::Str("hi".into())); }
    #[test] fn to_str_int() { assert_eq!(get(&run("let x: str = to_str(42);"), "x"), Value::Str("42".into())); }
    #[test] fn to_int_str() { assert_eq!(get(&run(r#"let x: int = to_int("99");"#), "x"), Value::Int(99)); }
    #[test] fn to_float_str(){ let v = f(get(&run(r#"let x: float = to_float("3.14");"#), "x")); assert!((v - 3.14).abs() < 1e-10); }
}
