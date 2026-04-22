use crate::math::Vec3;

#[derive(Debug, Clone)]
pub enum Expr {
    Primitive(Primitive),
    Variable(String),
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Argument>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Cube {
        width: f32,
        height: f32,
        depth: f32,
    },
    Sphere {
        radius: f32,
        segments: u32,
        rings: u32,
    },
    Cylinder {
        radius: f32,
        height: f32,
        segments: u32,
    },
    Plane {
        width: f32,
        depth: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub name: String,
    pub value: Value,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Integer(i64),
    String(String),
    Vec3(Vec3),
}

#[allow(dead_code)]
impl Value {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f32),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Integer(i) if *i >= 0 => Some(*i as u32),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            Value::Vec3(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expr,
    },
    Spawn(Expr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
