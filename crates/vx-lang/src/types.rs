use std::collections::HashMap;

use crate::ast::*;
use crate::error::LangError;

fn err(msg: impl Into<String>) -> LangError {
    LangError::ParseError { msg: msg.into(), offset: 0 }
}

/// Static type environment mapping names to their declared types.
#[derive(Debug, Default, Clone)]
pub struct TypeEnv {
    frames: Vec<HashMap<String, Ty>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self { frames: vec![HashMap::new()] }
    }

    pub fn push(&mut self) { self.frames.push(HashMap::new()); }
    pub fn pop(&mut self) { if self.frames.len() > 1 { self.frames.pop(); } }

    pub fn bind(&mut self, name: &str, ty: Ty) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), ty);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Ty> {
        for frame in self.frames.iter().rev() {
            if let Some(ty) = frame.get(name) { return Some(ty); }
        }
        None
    }
}

/// Infer the type of an expression given a type environment.
pub fn infer(expr: &Spanned<Expr>, env: &TypeEnv) -> Result<Ty, LangError> {
    match &expr.node {
        Expr::Lit(lit) => Ok(infer_lit(lit)),
        Expr::Ident(name) => env.lookup(name).cloned().ok_or_else(|| {
            LangError::ParseError { msg: format!("unknown variable '{name}'"), offset: expr.span.start }
        }),
        Expr::Unary { op, expr: inner } => {
            let ty = infer(inner, env)?;
            check_unary(op, &ty, inner.span.start)
        }
        Expr::Binary { op, lhs, rhs } => {
            let lt = infer(lhs, env)?;
            let rt = infer(rhs, env)?;
            check_binary(op, &lt, &rt, lhs.span.start)
        }
        Expr::Block(stmts) => {
            let mut inner_env = env.clone();
            inner_env.push();
            let mut last = Ty::Named("unit".into());
            for stmt in stmts {
                match &stmt.node {
                    Stmt::Let { name, ty, init } => {
                        let inferred = if let Some(e) = init { infer(e, &inner_env)? } else { Ty::Named("unit".into()) };
                        let resolved = ty.clone().unwrap_or(inferred);
                        inner_env.bind(name, resolved);
                    }
                    Stmt::Return(val) => {
                        last = val.as_ref().map(|e| infer(e, &inner_env)).transpose()?.unwrap_or(Ty::Named("unit".into()));
                    }
                    Stmt::Expr(e) => { last = infer(e, &inner_env)?; }
                    Stmt::Assign { .. } | Stmt::For { .. } => { last = Ty::Named("unit".into()); }
                }
            }
            inner_env.pop();
            Ok(last)
        }
        Expr::If { cond, then_block, else_block } => {
            let ct = infer(cond, env)?;
            if ct != Ty::Bool { return Err(err("if condition must be bool")); }
            let tt = infer(then_block, env)?;
            if let Some(eb) = else_block {
                let et = infer(eb, env)?;
                if tt != et { return Err(err(format!("if branches have different types: {tt:?} vs {et:?}"))); }
            }
            Ok(tt)
        }
        Expr::Call { callee, args } => {
            match infer(callee, env)? {
                Ty::Fn { params, ret } => {
                    if args.len() != params.len() {
                        return Err(err(format!("wrong number of arguments: expected {}, got {}", params.len(), args.len())));
                    }
                    for (arg, expected) in args.iter().zip(&params) {
                        let got = infer(arg, env)?;
                        if !types_compatible(&got, expected) {
                            return Err(err(format!("argument type mismatch: expected {expected:?}, got {got:?}")));
                        }
                    }
                    Ok(*ret)
                }
                other => Err(err(format!("cannot call a value of type {other:?}"))),
            }
        }
        Expr::Field { obj, name } => {
            let ty = infer(obj, env)?;
            infer_field(&ty, name, expr.span.start)
        }
        Expr::Index { obj, .. } => {
            let _ty = infer(obj, env)?;
            Ok(Ty::Named("unknown".into()))
        }
        Expr::Text2d(_) | Expr::Text3d(_) => Ok(Ty::Geo),
    }
}

fn infer_lit(lit: &Lit) -> Ty {
    match lit {
        Lit::Int(_) => Ty::Int,
        Lit::Float(_) => Ty::Float,
        Lit::Bool(_) => Ty::Bool,
        Lit::Str(_) => Ty::Str,
        Lit::Null => Ty::Named("null".into()),
    }
}

fn check_unary(op: &UnOp, ty: &Ty, offset: usize) -> Result<Ty, LangError> {
    match (op, ty) {
        (UnOp::Neg, Ty::Int) => Ok(Ty::Int),
        (UnOp::Neg, Ty::Float) => Ok(Ty::Float),
        (UnOp::Not, Ty::Bool) => Ok(Ty::Bool),
        _ => Err(LangError::ParseError {
            msg: format!("cannot apply {op:?} to {ty:?}"),
            offset,
        }),
    }
}

fn check_binary(op: &BinOp, lt: &Ty, rt: &Ty, offset: usize) -> Result<Ty, LangError> {
    match op {
        BinOp::Add => match (lt, rt) {
            (Ty::Int, Ty::Int) => Ok(Ty::Int),
            (Ty::Float, Ty::Float) | (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Float),
            (Ty::Str, Ty::Str) => Ok(Ty::Str),
            _ => Err(type_err(op, lt, rt, offset)),
        },
        BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => match (lt, rt) {
            (Ty::Int, Ty::Int) => Ok(Ty::Int),
            (Ty::Float, Ty::Float) | (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Float),
            _ => Err(type_err(op, lt, rt, offset)),
        },
        BinOp::Pow => {
            if is_numeric(lt) && is_numeric(rt) { Ok(Ty::Float) }
            else { Err(type_err(op, lt, rt, offset)) }
        }
        BinOp::Eq | BinOp::NotEq => Ok(Ty::Bool),
        BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
            if (is_numeric(lt) && is_numeric(rt)) || (lt == &Ty::Str && rt == &Ty::Str) {
                Ok(Ty::Bool)
            } else {
                Err(type_err(op, lt, rt, offset))
            }
        }
        BinOp::And | BinOp::Or => match (lt, rt) {
            (Ty::Bool, Ty::Bool) => Ok(Ty::Bool),
            _ => Err(type_err(op, lt, rt, offset)),
        },
        BinOp::Range => Ok(Ty::Named("range".into())),
    }
}

fn infer_field(ty: &Ty, name: &str, offset: usize) -> Result<Ty, LangError> {
    match (ty, name) {
        (Ty::Vec2, "x" | "y") => Ok(Ty::Float),
        (Ty::Vec3, "x" | "y" | "z") => Ok(Ty::Float),
        (Ty::Vec4, "x" | "y" | "z" | "w") => Ok(Ty::Float),
        (Ty::Color, "r" | "g" | "b" | "a") => Ok(Ty::Float),
        _ => Err(LangError::ParseError {
            msg: format!("no field '{name}' on {ty:?}"),
            offset,
        }),
    }
}

fn is_numeric(ty: &Ty) -> bool { matches!(ty, Ty::Int | Ty::Float) }

fn types_compatible(got: &Ty, expected: &Ty) -> bool {
    got == expected || (is_numeric(got) && is_numeric(expected))
}

fn type_err(op: &BinOp, lt: &Ty, rt: &Ty, offset: usize) -> LangError {
    LangError::ParseError {
        msg: format!("operator {op:?} cannot be applied to {lt:?} and {rt:?}"),
        offset,
    }
}

/// Type-check a complete module, binding all declarations into the env.
/// Pre-populate `env` with the types of all built-in functions and constants.
/// Call this before `check_module` so the type checker accepts built-in names.
pub fn register_builtin_types(env: &mut TypeEnv) {
    // constants
    env.bind("PI",  Ty::Float);
    env.bind("TAU", Ty::Float);
    env.bind("E",   Ty::Float);
    env.bind("INF", Ty::Float);

    // 1-arg math → float
    for name in ["sqrt", "floor", "ceil", "round", "sin", "cos", "tan", "log", "exp"] {
        env.bind(name, Ty::Fn { params: vec![Ty::Float], ret: Box::new(Ty::Float) });
    }
    // abs: numeric → same (model as float for type checking)
    env.bind("abs", Ty::Fn { params: vec![Ty::Float], ret: Box::new(Ty::Float) });

    // 2-arg math → float
    for name in ["pow", "atan2", "min", "max"] {
        env.bind(name, Ty::Fn { params: vec![Ty::Float, Ty::Float], ret: Box::new(Ty::Float) });
    }

    // 3-arg math → float
    for name in ["clamp", "lerp"] {
        env.bind(name, Ty::Fn {
            params: vec![Ty::Float, Ty::Float, Ty::Float],
            ret: Box::new(Ty::Float),
        });
    }

    // string functions
    env.bind("len",   Ty::Fn { params: vec![Ty::Str], ret: Box::new(Ty::Int) });
    for name in ["upper", "lower", "trim"] {
        env.bind(name, Ty::Fn { params: vec![Ty::Str], ret: Box::new(Ty::Str) });
    }
    // to_str accepts anything — use a named placeholder type
    env.bind("to_str",   Ty::Fn { params: vec![Ty::Named("any".into())], ret: Box::new(Ty::Str) });
    env.bind("to_int",   Ty::Fn { params: vec![Ty::Str], ret: Box::new(Ty::Int) });
    env.bind("to_float", Ty::Fn { params: vec![Ty::Str], ret: Box::new(Ty::Float) });
}

pub fn check_module(module: &Module, env: &mut TypeEnv) -> Result<(), LangError> {
    for decl in &module.decls {
        match &decl.node {
            Decl::Fn { name, params, ret_ty, body } => {
                let param_types: Vec<Ty> = params.iter().map(|p| p.ty.clone()).collect();
                let ret = ret_ty.clone().unwrap_or(Ty::Named("unit".into()));
                env.bind(name, Ty::Fn { params: param_types.clone(), ret: Box::new(ret.clone()) });
                let mut fn_env = env.clone();
                fn_env.push();
                for p in params { fn_env.bind(&p.name, p.ty.clone()); }
                let body_ty = infer(body, &fn_env)?;
                if !types_compatible(&body_ty, &ret) {
                    return Err(LangError::ParseError {
                        msg: format!("function '{name}' body type {body_ty:?} does not match declared return type {ret:?}"),
                        offset: decl.span.start,
                    });
                }
            }
            Decl::Let { name, ty, init } => {
                let inferred = infer(init, env)?;
                let resolved = ty.clone().unwrap_or(inferred);
                env.bind(name, resolved);
            }
            Decl::Stmt(stmt) => match &stmt.node {
                Stmt::Expr(e) => { infer(e, env)?; }
                Stmt::Assign { target, value, .. } => {
                    // Check rhs; target type check is best-effort
                    if let Expr::Ident(name) = &target.node {
                        if env.lookup(name).is_some() {
                            infer(value, env)?;
                        }
                    } else {
                        infer(value, env)?;
                    }
                }
                Stmt::For { iter, body, .. } => {
                    infer(iter, env)?;
                    infer(body, env)?;
                }
                Stmt::Return(val) => {
                    if let Some(e) = val { infer(e, env)?; }
                }
                Stmt::Let { name, ty, init } => {
                    if let Some(e) = init {
                        let inferred = infer(e, env)?;
                        let resolved = ty.clone().unwrap_or(inferred);
                        env.bind(name, resolved);
                    }
                }
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn check(src: &str) -> Result<(), LangError> {
        let module = Parser::new(src).unwrap().parse_module().unwrap();
        let mut env = TypeEnv::new();
        check_module(&module, &mut env)
    }

    #[test]
    fn int_let_ok() { check("let x: int = 42;").unwrap(); }

    #[test]
    fn float_let_ok() { check("let f: float = 1.5;").unwrap(); }

    #[test]
    fn str_let_ok() { check(r#"let s: str = "hi";"#).unwrap(); }

    #[test]
    fn bool_let_ok() { check("let b: bool = true;").unwrap(); }

    #[test]
    fn add_ints_ok() { check("let r: int = 1 + 2;").unwrap(); }

    #[test]
    fn add_strs_ok() { check(r#"let r: str = "a" + "b";"#).unwrap(); }

    #[test]
    fn fn_check_ok() { check("fn id(x: int) -> int { x; }").unwrap(); }
}
