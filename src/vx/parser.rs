use crate::vx::{Token, ast::*};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.advance() {
            Some(token) if std::mem::discriminant(&token) == std::mem::discriminant(&expected) => Ok(()),
            Some(token) => Err(format!("Expected {:?}, got {:?}", expected, token)),
            None => Err(format!("Expected {:?}, got EOF", expected)),
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while self.current().is_some() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::Spawn) => self.parse_spawn(),
            _ => Err(format!("Unexpected token: {:?}", self.current())),
        }
    }

    fn parse_let(&mut self) -> Result<Statement, String> {
        self.expect(Token::Let)?;

        let name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("Expected identifier after 'let'".to_string()),
        };

        self.expect(Token::Equals)?;

        let value = self.parse_expr()?;

        Ok(Statement::Let { name, value })
    }

    fn parse_spawn(&mut self) -> Result<Statement, String> {
        self.expect(Token::Spawn)?;
        self.expect(Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(Token::RParen)?;
        Ok(Statement::Spawn(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        while let Some(Token::Dot) = self.current() {
            self.advance();
            expr = self.parse_method_call(expr)?;
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(Token::Cube) => self.parse_cube(),
            Some(Token::Sphere) => self.parse_sphere(),
            Some(Token::Cylinder) => self.parse_cylinder(),
            Some(Token::Plane) => self.parse_plane(),
            Some(Token::Identifier(_)) => {
                let name = match self.advance() {
                    Some(Token::Identifier(name)) => name,
                    _ => unreachable!(),
                };

                if let Some(Token::LParen) = self.current() {
                    self.parse_function_call(name)
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::Union) | Some(Token::Difference) | Some(Token::Intersection) => {
                let func_name = match self.advance() {
                    Some(Token::Union) => "union".to_string(),
                    Some(Token::Difference) => "difference".to_string(),
                    Some(Token::Intersection) => "intersection".to_string(),
                    _ => unreachable!(),
                };
                self.parse_function_call(func_name)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", self.current())),
        }
    }

    fn parse_cube(&mut self) -> Result<Expr, String> {
        self.expect(Token::Cube)?;
        self.expect(Token::LParen)?;

        let mut width = 1.0;
        let mut height = 1.0;
        let mut depth = 1.0;

        while let Some(token) = self.current() {
            if matches!(token, Token::RParen) {
                break;
            }

            let param_name = match self.advance() {
                Some(Token::Width) => "width",
                Some(Token::Height) => "height",
                Some(Token::Depth) => "depth",
                _ => return Err("Expected parameter name".to_string()),
            };

            self.expect(Token::Colon)?;

            let value = match self.advance() {
                Some(Token::Float(f)) => f,
                Some(Token::Integer(i)) => i as f32,
                _ => return Err("Expected numeric value".to_string()),
            };

            match param_name {
                "width" => width = value,
                "height" => height = value,
                "depth" => depth = value,
                _ => {}
            }

            if let Some(Token::Comma) = self.current() {
                self.advance();
            }
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Primitive(Primitive::Cube { width, height, depth }))
    }

    fn parse_sphere(&mut self) -> Result<Expr, String> {
        self.expect(Token::Sphere)?;
        self.expect(Token::LParen)?;

        let mut radius = 1.0;
        let mut segments = 32;
        let mut rings = 16;

        while let Some(token) = self.current() {
            if matches!(token, Token::RParen) {
                break;
            }

            let param_name = match self.advance() {
                Some(Token::Radius) => "radius",
                Some(Token::Segments) => "segments",
                Some(Token::Identifier(name)) if name == "rings" => "rings",
                _ => return Err("Expected parameter name".to_string()),
            };

            self.expect(Token::Colon)?;

            match param_name {
                "radius" => {
                    radius = match self.advance() {
                        Some(Token::Float(f)) => f,
                        Some(Token::Integer(i)) => i as f32,
                        _ => return Err("Expected numeric value".to_string()),
                    };
                }
                "segments" => {
                    segments = match self.advance() {
                        Some(Token::Integer(i)) if i > 0 => i as u32,
                        _ => return Err("Expected positive integer".to_string()),
                    };
                }
                "rings" => {
                    rings = match self.advance() {
                        Some(Token::Integer(i)) if i > 0 => i as u32,
                        _ => return Err("Expected positive integer".to_string()),
                    };
                }
                _ => {}
            }

            if let Some(Token::Comma) = self.current() {
                self.advance();
            }
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Primitive(Primitive::Sphere { radius, segments, rings }))
    }

    fn parse_cylinder(&mut self) -> Result<Expr, String> {
        self.expect(Token::Cylinder)?;
        self.expect(Token::LParen)?;

        let mut radius = 1.0;
        let mut height = 2.0;
        let mut segments = 32;

        while let Some(token) = self.current() {
            if matches!(token, Token::RParen) {
                break;
            }

            let param_name = match self.advance() {
                Some(Token::Radius) => "radius",
                Some(Token::Height) => "height",
                Some(Token::Segments) => "segments",
                _ => return Err("Expected parameter name".to_string()),
            };

            self.expect(Token::Colon)?;

            match param_name {
                "radius" => {
                    radius = match self.advance() {
                        Some(Token::Float(f)) => f,
                        Some(Token::Integer(i)) => i as f32,
                        _ => return Err("Expected numeric value".to_string()),
                    };
                }
                "height" => {
                    height = match self.advance() {
                        Some(Token::Float(f)) => f,
                        Some(Token::Integer(i)) => i as f32,
                        _ => return Err("Expected numeric value".to_string()),
                    };
                }
                "segments" => {
                    segments = match self.advance() {
                        Some(Token::Integer(i)) if i > 0 => i as u32,
                        _ => return Err("Expected positive integer".to_string()),
                    };
                }
                _ => {}
            }

            if let Some(Token::Comma) = self.current() {
                self.advance();
            }
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Primitive(Primitive::Cylinder { radius, height, segments }))
    }

    fn parse_plane(&mut self) -> Result<Expr, String> {
        self.expect(Token::Plane)?;
        self.expect(Token::LParen)?;

        let mut width = 10.0;
        let mut depth = 10.0;

        while let Some(token) = self.current() {
            if matches!(token, Token::RParen) {
                break;
            }

            let param_name = match self.advance() {
                Some(Token::Width) => "width",
                Some(Token::Depth) => "depth",
                _ => return Err("Expected parameter name".to_string()),
            };

            self.expect(Token::Colon)?;

            let value = match self.advance() {
                Some(Token::Float(f)) => f,
                Some(Token::Integer(i)) => i as f32,
                _ => return Err("Expected numeric value".to_string()),
            };

            match param_name {
                "width" => width = value,
                "depth" => depth = value,
                _ => {}
            }

            if let Some(Token::Comma) = self.current() {
                self.advance();
            }
        }

        self.expect(Token::RParen)?;

        Ok(Expr::Primitive(Primitive::Plane { width, depth }))
    }

    fn parse_method_call(&mut self, object: Expr) -> Result<Expr, String> {
        let method = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(Token::SetMaterial) => "set_material".to_string(),
            Some(Token::Translate) => "translate".to_string(),
            Some(Token::Rotate) => "rotate".to_string(),
            Some(Token::Scale) => "scale".to_string(),
            _ => return Err("Expected method name".to_string()),
        };

        self.expect(Token::LParen)?;

        let args = self.parse_arguments()?;

        self.expect(Token::RParen)?;

        Ok(Expr::MethodCall {
            object: Box::new(object),
            method,
            args,
        })
    }

    fn parse_function_call(&mut self, name: String) -> Result<Expr, String> {
        self.expect(Token::LParen)?;

        let mut args = Vec::new();

        while let Some(token) = self.current() {
            if matches!(token, Token::RParen) {
                break;
            }

            args.push(self.parse_expr()?);

            if let Some(Token::Comma) = self.current() {
                self.advance();
            }
        }

        self.expect(Token::RParen)?;

        Ok(Expr::FunctionCall { name, args })
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, String> {
        let mut args = Vec::new();

        while let Some(token) = self.current() {
            if matches!(token, Token::RParen) {
                break;
            }

            let name = match self.advance() {
                Some(Token::ColorParam) => "color".to_string(),
                Some(Token::Roughness) => "roughness".to_string(),
                Some(Token::Metallic) => "metallic".to_string(),
                Some(Token::X) => "x".to_string(),
                Some(Token::Y) => "y".to_string(),
                Some(Token::Z) => "z".to_string(),
                Some(Token::Identifier(name)) => name,
                _ => return Err("Expected argument name".to_string()),
            };

            self.expect(Token::Colon)?;

            let value = match self.advance() {
                Some(Token::Float(f)) => Value::Float(f),
                Some(Token::Integer(i)) => Value::Integer(i),
                Some(Token::String(s)) => Value::String(s),
                _ => return Err("Expected argument value".to_string()),
            };

            args.push(Argument { name, value });

            if let Some(Token::Comma) = self.current() {
                self.advance();
            }
        }

        Ok(args)
    }
}
