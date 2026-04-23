use logos::Logos;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LexError;

/// Every token in the VX source language.
///
/// Multi-character operators are matched before their single-character
/// prefixes so logos always picks the longest match first.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = LexError)]
#[logos(skip r"[ \t\r\n\f]+")] // whitespace
#[logos(skip r"//[^\n]*")] // line comments
#[logos(skip r"/\*([^*]|\*[^/])*\*/")] // block comments (non-nested)
pub enum Token<'s> {
    // ── keywords ──────────────────────────────────────────────────────────
    #[token("let")]
    Let,
    #[token("fn")]
    Fn,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    // ── built-in type names ───────────────────────────────────────────────
    #[token("int")]
    TyInt,
    #[token("float")]
    TyFloat,
    #[token("bool")]
    TyBool,
    #[token("str")]
    TyStr,
    #[token("vec2")]
    TyVec2,
    #[token("vec3")]
    TyVec3,
    #[token("vec4")]
    TyVec4,
    #[token("mat4")]
    TyMat4,
    #[token("color")]
    TyColor,
    #[token("geo")]
    TyGeo,
    // ── scene-domain keywords ─────────────────────────────────────────────
    #[token("mesh")]
    Mesh,
    #[token("material")]
    Material,
    #[token("scene")]
    Scene,
    #[token("object")]
    Object,
    #[token("light")]
    Light,
    #[token("camera")]
    Camera,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("text2d")]
    Text2d,
    #[token("text3d")]
    Text3d,
    // ── literals ──────────────────────────────────────────────────────────
    /// Float must come before integer so `1.0` is not lexed as `1` `.` `0`.
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice())]
    Float(&'s str),

    #[regex(r"[0-9]+", |lex| lex.slice())]
    Int(&'s str),

    /// Raw slice including surrounding quotes; parser strips them.
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice())]
    Str(&'s str),

    // ── identifier (after all keyword tokens) ─────────────────────────────
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Ident(&'s str),

    // ── compound operators (longest match first) ──────────────────────────
    #[token("**")]
    StarStar,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("->")]
    Arrow,
    #[token("..")]
    DotDot,
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    // ── single-character operators ────────────────────────────────────────
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("!")]
    Bang,
    #[token("=")]
    Eq,
    #[token(".")]
    Dot,
    // ── delimiters ────────────────────────────────────────────────────────
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
}

/// A token together with its byte-range in the source.
pub type Spanned<'s> = (Result<Token<'s>, LexError>, logos::Span);

/// Lex `src` and return an iterator of `(Result<Token, LexError>, Span)`.
pub fn lex(src: &str) -> impl Iterator<Item = Spanned<'_>> {
    Token::lexer(src).spanned()
}

/// Collect all tokens, returning `Err` on the first lex error.
pub fn lex_all(src: &str) -> Result<Vec<(Token<'_>, logos::Span)>, (LexError, logos::Span)> {
    Token::lexer(src)
        .spanned()
        .map(|(res, span)| res.map(|tok| (tok, span.clone())).map_err(|e| (e, span)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(src: &str) -> Vec<Token<'_>> {
        Token::lexer(src)
            .collect::<Result<Vec<_>, _>>()
            .expect("lex should succeed")
    }

    fn first(src: &str) -> Token<'_> {
        tokens(src).into_iter().next().expect("at least one token")
    }

    #[test]
    fn keywords() {
        assert_eq!(first("let"), Token::Let);
        assert_eq!(first("fn"), Token::Fn);
        assert_eq!(first("for"), Token::For);
        assert_eq!(first("in"), Token::In);
        assert_eq!(first("if"), Token::If);
        assert_eq!(first("else"), Token::Else);
        assert_eq!(first("return"), Token::Return);
        assert_eq!(first("true"), Token::True);
        assert_eq!(first("false"), Token::False);
        assert_eq!(first("null"), Token::Null);
    }

    #[test]
    fn type_keywords() {
        assert_eq!(first("int"), Token::TyInt);
        assert_eq!(first("float"), Token::TyFloat);
        assert_eq!(first("bool"), Token::TyBool);
        assert_eq!(first("str"), Token::TyStr);
        assert_eq!(first("vec2"), Token::TyVec2);
        assert_eq!(first("vec3"), Token::TyVec3);
        assert_eq!(first("vec4"), Token::TyVec4);
        assert_eq!(first("mat4"), Token::TyMat4);
        assert_eq!(first("color"), Token::TyColor);
        assert_eq!(first("geo"), Token::TyGeo);
    }

    #[test]
    fn scene_keywords() {
        assert_eq!(first("mesh"), Token::Mesh);
        assert_eq!(first("material"), Token::Material);
        assert_eq!(first("scene"), Token::Scene);
        assert_eq!(first("object"), Token::Object);
        assert_eq!(first("light"), Token::Light);
        assert_eq!(first("camera"), Token::Camera);
        assert_eq!(first("import"), Token::Import);
        assert_eq!(first("export"), Token::Export);
    }

    #[test]
    fn integer_literal() {
        assert_eq!(first("42"), Token::Int("42"));
        assert_eq!(first("0"), Token::Int("0"));
        assert_eq!(first("1000000"), Token::Int("1000000"));
    }

    #[test]
    fn float_literal() {
        assert_eq!(first("3.14"), Token::Float("3.14"));
        assert_eq!(first("0.0"), Token::Float("0.0"));
        assert_eq!(first("1.5e10"), Token::Float("1.5e10"));
        assert_eq!(first("2.0E-3"), Token::Float("2.0E-3"));
    }

    #[test]
    fn float_before_int() {
        let toks = tokens("1.0");
        assert_eq!(toks, vec![Token::Float("1.0")]);
    }

    #[test]
    fn string_literal() {
        assert_eq!(first(r#""hello""#), Token::Str(r#""hello""#));
        assert_eq!(first(r#""with \"escape\"""#), Token::Str(r#""with \"escape\"""#));
        assert_eq!(first(r#""""#), Token::Str(r#""""#));
    }

    #[test]
    fn identifier() {
        assert_eq!(first("foo"), Token::Ident("foo"));
        assert_eq!(first("_bar"), Token::Ident("_bar"));
        assert_eq!(first("baz42"), Token::Ident("baz42"));
        assert_eq!(first("__x"), Token::Ident("__x"));
    }

    #[test]
    fn keyword_not_ident() {
        assert_ne!(first("let"), Token::Ident("let"));
        assert_ne!(first("fn"), Token::Ident("fn"));
    }

    #[test]
    fn ident_with_keyword_prefix() {
        assert_eq!(first("letter"), Token::Ident("letter"));
        assert_eq!(first("function"), Token::Ident("function"));
        assert_eq!(first("forall"), Token::Ident("forall"));
    }

    #[test]
    fn compound_operators() {
        assert_eq!(first("**"), Token::StarStar);
        assert_eq!(first("+="), Token::PlusEq);
        assert_eq!(first("-="), Token::MinusEq);
        assert_eq!(first("*="), Token::StarEq);
        assert_eq!(first("/="), Token::SlashEq);
        assert_eq!(first("=="), Token::EqEq);
        assert_eq!(first("!="), Token::BangEq);
        assert_eq!(first("<="), Token::LtEq);
        assert_eq!(first(">="), Token::GtEq);
        assert_eq!(first("->"), Token::Arrow);
        assert_eq!(first(".."), Token::DotDot);
        assert_eq!(first("&&"), Token::AmpAmp);
        assert_eq!(first("||"), Token::PipePipe);
    }

    #[test]
    fn single_operators() {
        assert_eq!(first("+"), Token::Plus);
        assert_eq!(first("-"), Token::Minus);
        assert_eq!(first("*"), Token::Star);
        assert_eq!(first("/"), Token::Slash);
        assert_eq!(first("%"), Token::Percent);
        assert_eq!(first("<"), Token::Lt);
        assert_eq!(first(">"), Token::Gt);
        assert_eq!(first("!"), Token::Bang);
        assert_eq!(first("="), Token::Eq);
        assert_eq!(first("."), Token::Dot);
    }

    #[test]
    fn delimiters() {
        assert_eq!(first("("), Token::LParen);
        assert_eq!(first(")"), Token::RParen);
        assert_eq!(first("{"), Token::LBrace);
        assert_eq!(first("}"), Token::RBrace);
        assert_eq!(first("["), Token::LBracket);
        assert_eq!(first("]"), Token::RBracket);
        assert_eq!(first(","), Token::Comma);
        assert_eq!(first(";"), Token::Semi);
        assert_eq!(first(":"), Token::Colon);
    }

    #[test]
    fn whitespace_skipped() {
        let toks = tokens("  let   x  ");
        assert_eq!(toks, vec![Token::Let, Token::Ident("x")]);
    }

    #[test]
    fn line_comment_skipped() {
        let toks = tokens("let // this is a comment\nx");
        assert_eq!(toks, vec![Token::Let, Token::Ident("x")]);
    }

    #[test]
    fn block_comment_skipped() {
        let toks = tokens("let /* block */ x");
        assert_eq!(toks, vec![Token::Let, Token::Ident("x")]);
    }

    #[test]
    fn full_let_statement() {
        let toks = tokens("let x: int = 42;");
        assert_eq!(
            toks,
            vec![
                Token::Let,
                Token::Ident("x"),
                Token::Colon,
                Token::TyInt,
                Token::Eq,
                Token::Int("42"),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn full_fn_signature() {
        let toks = tokens("fn add(a: float, b: float) -> float");
        assert_eq!(
            toks,
            vec![
                Token::Fn,
                Token::Ident("add"),
                Token::LParen,
                Token::Ident("a"),
                Token::Colon,
                Token::TyFloat,
                Token::Comma,
                Token::Ident("b"),
                Token::Colon,
                Token::TyFloat,
                Token::RParen,
                Token::Arrow,
                Token::TyFloat,
            ]
        );
    }

    #[test]
    fn star_star_not_two_stars() {
        let toks = tokens("2**3");
        assert_eq!(
            toks,
            vec![Token::Int("2"), Token::StarStar, Token::Int("3")]
        );
    }

    #[test]
    fn unknown_character_errors() {
        let result = lex_all("@");
        assert!(result.is_err());
    }

    #[test]
    fn span_positions() {
        let spanned: Vec<_> = lex("let x").collect();
        let (tok0, span0) = &spanned[0];
        let (tok1, span1) = &spanned[1];
        assert_eq!(tok0.as_ref().unwrap(), &Token::Let);
        assert_eq!(span0.start, 0);
        assert_eq!(span0.end, 3);
        assert_eq!(tok1.as_ref().unwrap(), &Token::Ident("x"));
        assert_eq!(span1.start, 4);
        assert_eq!(span1.end, 5);
    }
}
