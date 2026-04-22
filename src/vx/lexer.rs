use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
pub enum Token {
    #[token("let")]
    Let,

    #[token("cube")]
    Cube,

    #[token("sphere")]
    Sphere,

    #[token("cylinder")]
    Cylinder,

    #[token("plane")]
    Plane,

    #[token("union")]
    Union,

    #[token("difference")]
    Difference,

    #[token("intersection")]
    Intersection,

    #[token("spawn")]
    Spawn,

    #[token("translate")]
    Translate,

    #[token("rotate")]
    Rotate,

    #[token("scale")]
    Scale,

    #[token("set_material")]
    SetMaterial,

    #[token("color")]
    ColorParam,

    #[token("roughness")]
    Roughness,

    #[token("metallic")]
    Metallic,

    #[token("width")]
    Width,

    #[token("height")]
    Height,

    #[token("depth")]
    Depth,

    #[token("radius")]
    Radius,

    #[token("segments")]
    Segments,

    #[token("x")]
    X,

    #[token("y")]
    Y,

    #[token("z")]
    Z,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 1, callback = |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f32>().ok())]
    Float(f32),

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token("=")]
    Equals,

    #[token(".")]
    Dot,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(input);

    while let Some(token) = lexer.next() {
        match token {
            Ok(t) => tokens.push(t),
            Err(_) => return Err(format!("Unexpected token at position {}", lexer.span().start)),
        }
    }

    Ok(tokens)
}
