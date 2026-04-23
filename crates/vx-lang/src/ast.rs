/// A value carrying its source-code byte range.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: std::ops::Range<usize>,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: std::ops::Range<usize>) -> Self {
        Self { node, span }
    }
}

/// VX built-in types.
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
    Color,
    Geo,
    Named(String),
    Fn { params: Vec<Ty>, ret: Box<Ty> },
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Range,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

/// Assignment operators.
#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// Expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit),
    Ident(String),
    Binary {
        op: BinOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },
    Unary {
        op: UnOp,
        expr: Box<Spanned<Expr>>,
    },
    Call {
        callee: Box<Spanned<Expr>>,
        args: Vec<Spanned<Expr>>,
    },
    Index {
        obj: Box<Spanned<Expr>>,
        idx: Box<Spanned<Expr>>,
    },
    Field {
        obj: Box<Spanned<Expr>>,
        name: String,
    },
    Block(Vec<Spanned<Stmt>>),
    If {
        cond: Box<Spanned<Expr>>,
        then_block: Box<Spanned<Expr>>,
        else_block: Option<Box<Spanned<Expr>>>,
    },
    Text2d(Box<TextProps>),
    Text3d(Box<TextProps>),
}

/// Statements.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Ty>,
        init: Option<Spanned<Expr>>,
    },
    Assign {
        target: Spanned<Expr>,
        op: AssignOp,
        value: Spanned<Expr>,
    },
    Return(Option<Spanned<Expr>>),
    For {
        var: String,
        iter: Spanned<Expr>,
        body: Box<Spanned<Expr>>,
    },
    Expr(Spanned<Expr>),
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
    pub span: std::ops::Range<usize>,
}

/// Property block for a text declaration.
/// Every key maps to the expression that provides its value.
///
/// ### Recognised keys — both `text2d` and `text3d`
/// | Key | Type | Description |
/// |---|---|---|
/// | `content` | `str` | The text string to render |
/// | `font` | `str` | Font family name, e.g. `"Inter"` |
/// | `size` | `float` | Font size (points for 2-D, world-units for 3-D) |
/// | `color` | `color` | RGBA fill colour |
/// | `align` | `str` | `"left"` \| `"center"` \| `"right"` |
/// | `position` | `vec3` | World-space position |
/// | `italic` | `bool` | Italic style |
/// | `bold` | `bool` | Bold weight |
/// | `tracking` | `float` | Letter spacing multiplier (1.0 = normal) |
/// | `line_height` | `float` | Line-height multiplier (1.0 = normal) |
/// | `wrap_width` | `float` | Word-wrap width in world units; 0 = no wrap |
///
/// ### Additional keys — `text3d` only
/// | Key | Type | Description |
/// |---|---|---|
/// | `depth` | `float` | Extrusion depth along Z |
/// | `bevel_depth` | `float` | Bevel radius on extruded edges |
/// | `bevel_resolution` | `int` | Bevel curve segments (1–16) |
/// | `rotation` | `vec3` | Euler rotation in radians |
#[derive(Debug, Clone, PartialEq)]
pub struct TextProps {
    pub fields: std::collections::HashMap<String, Spanned<Expr>>,
}

/// Top-level declarations.
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Fn {
        name: String,
        params: Vec<Param>,
        ret_ty: Option<Ty>,
        body: Spanned<Expr>,
    },
    Let {
        name: String,
        ty: Option<Ty>,
        init: Spanned<Expr>,
    },
    /// A bare statement at module scope (for loops, assignments, expressions).
    Stmt(Spanned<Stmt>),
}

/// Root of a parsed VX source file.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub decls: Vec<Spanned<Decl>>,
}
