use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Node {
    VarDecl {
        is_pub: bool,
        is_mut: bool,
        name: String,
        ty: Option<Box<Node>>,
        value: Option<Box<Node>>,
    },
    FnDecl {
        is_pub: bool,
        receiver: Option<Box<Node>>,
        name: String,
        params: Vec<(String, Node)>,
        return_type: Option<Box<Node>>,
        body: Option<Box<Node>>,
    },
    TypeDecl {
        is_pub: bool,
        name: String,
        fields: Vec<(String, Node, Option<String>)>,
    },
    UnionDecl {
        is_pub: bool,
        name: String,
        variants: Vec<(String, Vec<Node>)>,
    },
    EnumDecl {
        is_pub: bool,
        name: String,
        variants: Vec<String>,
    },
    TestBlock(Box<Node>),

    Block(Vec<Node>),
    If {
        condition: Box<Node>,
        then_body: Box<Node>,
        elif_parts: Vec<(Box<Node>, Box<Node>)>,
        else_body: Option<Box<Node>>,
    },
    For {
        pattern: Option<String>,
        iterable: Option<Box<Node>>,
        body: Box<Node>,
    },
    Return(Option<Box<Node>>),
    Break,
    Match {
        expr: Box<Node>,
        arms: Vec<(String, Vec<String>, Box<Node>)>,
    },
    Assert(Box<Node>),

    Binary {
        op: String,
        left: Box<Node>,
        right: Box<Node>,
    },
    Unary {
        op: String,
        expr: Box<Node>,
    },
    Call {
        func: Box<Node>,
        args: Vec<Node>,
    },
    Index {
        expr: Box<Node>,
        index: Box<Node>,
    },
    Field {
        expr: Box<Node>,
        field: String,
    },
    Ident(String),
    Number(String),
    String(String),
    RawString(String),
    List(Vec<Node>),
    Struct(Vec<(String, Node)>),
    Cast {
        expr: Box<Node>,
        ty: Box<Node>,
    },
    Comment(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            Some(&self.tokens[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        if self.pos + offset < self.tokens.len() {
            Some(&self.tokens[self.pos + offset])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some(token) = self.current() {
            if std::mem::discriminant(token) == std::mem::discriminant(expected) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        if self.match_token(&expected) {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current()))
        }
    }

    fn skip_comments(&mut self) {
        while let Some(Token::Comment(_)) = self.current() {
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();

        loop {
            while let Some(Token::Comment(c)) = self.current() {
                let c = c.clone();
                self.advance();
                nodes.push(Node::Comment(c));
            }
            if self.current() == Some(&Token::Eof) {
                break;
            }

            if let Ok(node) = self.parse_top_level() {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    fn parse_top_level(&mut self) -> Result<Node, String> {
        self.skip_comments();

        let is_pub = if self.match_token(&Token::Pub) {
            true
        } else {
            false
        };

        match self.current() {
            Some(Token::Val) => self.parse_var_decl(is_pub),
            Some(Token::Fn) => self.parse_fn_decl(is_pub),
            Some(Token::Type) => self.parse_type_decl(is_pub),
            Some(Token::Union) => self.parse_union_decl(is_pub),
            Some(Token::Enum) => self.parse_enum_decl(is_pub),
            Some(Token::Test) => self.parse_test_block(),
            Some(Token::Extern) => self.parse_extern_fn(is_pub),
            Some(Token::When) => self.parse_platform_conditional(),
            _ => {
                self.pos += 1;
                Err("Unexpected token at top level".to_string())
            }
        }
    }

    fn parse_var_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        self.expect_token(Token::Val)?;

        let is_mut = if self.match_token(&Token::Mut) {
            true
        } else {
            false
        };

        let name = if let Some(Token::Ident(n)) = self.current() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err("Expected identifier after val".to_string());
        };

        let ty = if self.match_token(&Token::Colon) {
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };

        let value = if self.match_token(&Token::Equal) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.match_token(&Token::Semicolon);

        Ok(Node::VarDecl {
            is_pub,
            is_mut,
            name,
            ty,
            value,
        })
    }

    fn parse_fn_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        self.expect_token(Token::Fn)?;

        let receiver = if self.match_token(&Token::LParen) {
            let receiver_type = self.parse_type()?;
            self.expect_token(Token::RParen)?;
            Some(Box::new(receiver_type))
        } else {
            None
        };

        let name = if let Some(Token::Ident(n)) = self.current() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err("Expected function name".to_string());
        };

        self.expect_token(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect_token(Token::RParen)?;

        let return_type = if self.current() != Some(&Token::Equal)
            && self.current() != Some(&Token::LBrace)
        {
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };

        let body = if self.match_token(&Token::Equal) {
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };

        Ok(Node::FnDecl {
            is_pub,
            receiver,
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Node)>, String> {
        let mut params = Vec::new();

        while self.current() != Some(&Token::RParen) && self.current() != Some(&Token::Eof) {
            if !params.is_empty() {
                self.expect_token(Token::Comma)?;
            }

            let name = if let Some(Token::Ident(n)) = self.current() {
                let n = n.clone();
                self.advance();
                n
            } else {
                return Err("Expected parameter name".to_string());
            };

            self.expect_token(Token::Colon)?;
            let ty = self.parse_type()?;

            params.push((name, ty));
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Node, String> {
        let base = if let Some(Token::Ident(name)) = self.current() {
            if name == "ref" {
                self.advance();
                let ty = self.parse_type()?;
                let ty_str = self.node_type_to_string(&ty);
                Node::Ident(format!("ref {}", ty_str))
            } else {
                let n = name.clone();
                self.advance();
                Node::Ident(n)
            }
        } else if self.match_token(&Token::Mut) {
            let ty = self.parse_type()?;
            let ty_str = self.node_type_to_string(&ty);
            Node::Ident(format!("mut {}", ty_str))
        } else if self.match_token(&Token::Ampersand) {
            let ty = self.parse_type()?;
            let ty_str = self.node_type_to_string(&ty);
            Node::Ident(format!("ref {}", ty_str))
        } else {
            return Err("Expected type".to_string());
        };

        if self.match_token(&Token::LBracket) {
            let inner = self.parse_type()?;
            self.expect_token(Token::RBracket)?;
            let inner_str = self.node_type_to_string(&inner);
            return Ok(Node::Ident(format!("list[{}]", inner_str)));
        }

        if self.match_token(&Token::Bang) {
            let base_str = self.node_type_to_string(&base);
            return Ok(Node::Ident(format!("{}!", base_str)));
        }

        Ok(base)
    }

    fn node_type_to_string(&self, node: &Node) -> String {
        match node {
            Node::Ident(name) => name.clone(),
            _ => format!("{:?}", node),
        }
    }

    fn parse_type_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        self.expect_token(Token::Type)?;

        let name = if let Some(Token::Ident(n)) = self.current() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err("Expected type name".to_string());
        };

        self.expect_token(Token::Equal)?;
        self.expect_token(Token::LBrace)?;

        let mut fields = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            self.skip_comments();
            if self.current() == Some(&Token::RBrace) {
                break;
            }

            let field_name = if let Some(Token::Ident(n)) = self.current() {
                let n = n.clone();
                self.advance();
                n
            } else {
                break;
            };

            self.expect_token(Token::Colon)?;
            let field_type = self.parse_type()?;

            // Comma may appear before inline comment (field: type, // comment)
            // or comment may appear directly after type (field: type // comment)
            let had_comma = self.match_token(&Token::Comma);

            let inline_comment = if let Some(Token::Comment(c)) = self.current() {
                let c = c.clone();
                self.advance();
                Some(c)
            } else {
                None
            };

            fields.push((field_name, field_type, inline_comment));

            if !had_comma {
                if self.current() == Some(&Token::RBrace) {
                    // done
                } else {
                    // try consuming a comma if present (no-comma style)
                    if !self.match_token(&Token::Comma) && self.current() != Some(&Token::RBrace) {
                        break;
                    }
                }
            }
        }

        self.expect_token(Token::RBrace)?;

        Ok(Node::TypeDecl {
            is_pub,
            name,
            fields,
        })
    }

    fn parse_union_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        self.expect_token(Token::Union)?;

        let name = if let Some(Token::Ident(n)) = self.current() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err("Expected union name".to_string());
        };

        self.expect_token(Token::Equal)?;
        self.expect_token(Token::LBrace)?;

        let mut variants = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            let variant_name = if let Some(Token::Ident(n)) = self.current() {
                let n = n.clone();
                self.advance();
                n
            } else {
                break;
            };

            let mut variant_types = Vec::new();

            if self.match_token(&Token::LParen) {
                while self.current() != Some(&Token::RParen) {
                    if !variant_types.is_empty() {
                        self.match_token(&Token::Comma);
                    }
                    variant_types.push(self.parse_type()?);
                }
                self.expect_token(Token::RParen)?;
            }

            variants.push((variant_name, variant_types));

            if !self.match_token(&Token::Pipe) {
                break;
            }
        }

        self.expect_token(Token::RBrace)?;

        Ok(Node::UnionDecl {
            is_pub,
            name,
            variants,
        })
    }

    fn parse_enum_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        self.expect_token(Token::Enum)?;

        let name = if let Some(Token::Ident(n)) = self.current() {
            let n = n.clone();
            self.advance();
            n
        } else {
            return Err("Expected enum name".to_string());
        };

        self.expect_token(Token::Equal)?;
        self.expect_token(Token::LBrace)?;

        let mut variants = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            if let Some(Token::Ident(n)) = self.current() {
                variants.push(n.clone());
                self.advance();
            } else {
                break;
            }

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.expect_token(Token::RBrace)?;

        Ok(Node::EnumDecl {
            is_pub,
            name,
            variants,
        })
    }

    fn parse_test_block(&mut self) -> Result<Node, String> {
        self.expect_token(Token::Test)?;
        self.expect_token(Token::LBrace)?;

        let mut statements = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            self.skip_comments();
            if self.current() == Some(&Token::RBrace) {
                break;
            }

            statements.push(self.parse_statement()?);
        }

        self.expect_token(Token::RBrace)?;

        Ok(Node::TestBlock(Box::new(Node::Block(statements))))
    }

    fn parse_extern_fn(&mut self, is_pub: bool) -> Result<Node, String> {
        self.expect_token(Token::Extern)?;
        self.expect_token(Token::LParen)?;
        if let Some(Token::Ident(_)) = self.current() {
            self.advance();
        }
        self.expect_token(Token::RParen)?;

        self.parse_fn_decl(is_pub)
    }

    fn parse_platform_conditional(&mut self) -> Result<Node, String> {
        self.expect_token(Token::When)?;

        let platform_str = match self.current() {
            Some(Token::Linux) => {
                self.advance();
                "linux".to_string()
            }
            Some(Token::Darwin) => {
                self.advance();
                "darwin".to_string()
            }
            Some(Token::Windows) => {
                self.advance();
                "windows".to_string()
            }
            Some(Token::Posix) => {
                self.advance();
                "posix".to_string()
            }
            _ => {
                if let Some(Token::Ident(p)) = self.current() {
                    let p_copy = p.clone();
                    self.advance();
                    p_copy
                } else {
                    return Err("Expected platform name".to_string());
                }
            }
        };

        self.expect_token(Token::LBrace)?;

        let mut statements = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            self.skip_comments();
            if self.current() == Some(&Token::RBrace) {
                break;
            }

            statements.push(self.parse_top_level()?);
        }

        self.expect_token(Token::RBrace)?;

        Ok(Node::Block(statements))
    }

    fn parse_statement(&mut self) -> Result<Node, String> {
        self.skip_comments();

        match self.current() {
            Some(Token::Val) => self.parse_var_decl(false),
            Some(Token::If) => self.parse_if(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Break) => {
                self.advance();
                self.match_token(&Token::Semicolon);
                Ok(Node::Break)
            }
            Some(Token::Assert) => self.parse_assert(),
            Some(Token::Match) => self.parse_match(),
            Some(Token::LBrace) => self.parse_block(),
            _ => {
                let expr = self.parse_expr()?;
                let op = match self.current() {
                    Some(Token::Equal) => Some("="),
                    Some(Token::PlusEq) => Some("+="),
                    Some(Token::MinusEq) => Some("-="),
                    Some(Token::StarEq) => Some("*="),
                    Some(Token::SlashEq) => Some("/="),
                    _ => None,
                };
                if let Some(op_str) = op {
                    let op_str = op_str.to_string();
                    self.advance();
                    let value = self.parse_expr()?;
                    self.match_token(&Token::Semicolon);
                    Ok(Node::Binary {
                        op: op_str,
                        left: Box::new(expr),
                        right: Box::new(value),
                    })
                } else {
                    self.match_token(&Token::Semicolon);
                    Ok(expr)
                }
            }
        }
    }

    fn parse_block(&mut self) -> Result<Node, String> {
        self.expect_token(Token::LBrace)?;

        let mut statements = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            if let Some(Token::Comment(c)) = self.current() {
                let c = c.clone();
                self.advance();
                statements.push(Node::Comment(c));
                continue;
            }
            if self.current() == Some(&Token::RBrace) {
                break;
            }

            statements.push(self.parse_statement()?);
        }

        self.expect_token(Token::RBrace)?;

        Ok(Node::Block(statements))
    }

    fn parse_if(&mut self) -> Result<Node, String> {
        self.expect_token(Token::If)?;

        let condition = Box::new(self.parse_expr()?);
        let then_body = Box::new(self.parse_block()?);

        let mut elif_parts = Vec::new();
        while self.match_token(&Token::Elif) {
            let elif_cond = Box::new(self.parse_expr()?);
            let elif_body = Box::new(self.parse_block()?);
            elif_parts.push((elif_cond, elif_body));
        }

        let else_body = if self.match_token(&Token::Else) {
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };

        Ok(Node::If {
            condition,
            then_body,
            elif_parts,
            else_body,
        })
    }

    fn parse_for(&mut self) -> Result<Node, String> {
        self.expect_token(Token::For)?;

        let (pattern, iterable) = if self.current() == Some(&Token::LBrace) {
            (None, None)
        } else if let Some(Token::Ident(name)) = self.current() {
            let n = name.clone();
            self.advance();

            if self.match_token(&Token::In) {
                let iter = self.parse_expr()?;
                (Some(n), Some(Box::new(iter)))
            } else {
                (Some(n), None)
            }
        } else {
            (None, None)
        };

        let body = Box::new(self.parse_block()?);

        Ok(Node::For {
            pattern,
            iterable,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Node, String> {
        self.expect_token(Token::Return)?;

        let value = if self.current() == Some(&Token::Semicolon)
            || self.current() == Some(&Token::RBrace)
        {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };

        self.match_token(&Token::Semicolon);

        Ok(Node::Return(value))
    }

    fn parse_match(&mut self) -> Result<Node, String> {
        self.expect_token(Token::Match)?;
        let expr = Box::new(self.parse_expr()?);
        self.expect_token(Token::LBrace)?;

        let mut arms = Vec::new();

        while self.current() != Some(&Token::RBrace) && self.current() != Some(&Token::Eof) {
            self.skip_comments();
            if self.current() == Some(&Token::RBrace) {
                break;
            }

            let (pattern, bindings) = self.parse_match_pattern()?;
            self.expect_token(Token::Arrow)?;
            let body = Box::new(self.parse_block()?);
            arms.push((pattern, bindings, body));
        }

        self.expect_token(Token::RBrace)?;
        Ok(Node::Match { expr, arms })
    }

    fn parse_match_pattern(&mut self) -> Result<(String, Vec<String>), String> {
        let mut pattern = String::new();
        let mut bindings = Vec::new();

        match self.current() {
            Some(Token::Some) => {
                pattern.push_str("some");
                self.advance();
                if let Some(Token::Ident(bind)) = self.current() {
                    let b = bind.clone();
                    pattern.push(' ');
                    pattern.push_str(&b);
                    bindings.push(b);
                    self.advance();
                }
            }
            Some(Token::None) => {
                pattern.push_str("none");
                self.advance();
            }
            Some(Token::Ok) => {
                pattern.push_str("ok");
                self.advance();
                if let Some(Token::Ident(bind)) = self.current() {
                    let b = bind.clone();
                    pattern.push(' ');
                    pattern.push_str(&b);
                    bindings.push(b);
                    self.advance();
                }
            }
            Some(Token::Err) => {
                pattern.push_str("err");
                self.advance();
                if let Some(Token::Ident(bind)) = self.current() {
                    let b = bind.clone();
                    pattern.push(' ');
                    pattern.push_str(&b);
                    bindings.push(b);
                    self.advance();
                }
            }
            Some(Token::Ident(name)) => {
                let n = name.clone();
                pattern.push_str(&n);
                self.advance();
            }
            _ => return Err(format!("Expected match pattern, got {:?}", self.current())),
        }

        Ok((pattern, bindings))
    }

    fn parse_assert(&mut self) -> Result<Node, String> {
        self.expect_token(Token::Assert)?;

        let expr = Box::new(self.parse_expr()?);
        self.match_token(&Token::Semicolon);

        Ok(Node::Assert(expr))
    }

    fn parse_expr(&mut self) -> Result<Node, String> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: i32) -> Result<Node, String> {
        let mut left = self.parse_unary_expr()?;

        while let Some(op_token) = self.current() {
            let prec = self.get_precedence(op_token);
            if prec < min_prec {
                break;
            }

            let op = self.token_to_op(op_token)?;
            self.advance();

            let right = self.parse_binary_expr(prec + 1)?;
            left = Node::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Node, String> {
        match self.current() {
            Some(Token::Minus) | Some(Token::Bang) | Some(Token::Tilde) => {
                let op = self.token_to_op(self.current().unwrap())?;
                self.advance();
                let expr = self.parse_unary_expr()?;
                Ok(Node::Unary {
                    op,
                    expr: Box::new(expr),
                })
            }
            Some(Token::Trust) => {
                self.advance();
                self.parse_unary_expr()
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Node, String> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            match self.current() {
                Some(Token::LParen) => {
                    self.advance();
                    let mut args = Vec::new();

                    while self.current() != Some(&Token::RParen)
                        && self.current() != Some(&Token::Eof)
                    {
                        if !args.is_empty() {
                            self.expect_token(Token::Comma)?;
                        }
                        args.push(self.parse_expr()?);
                    }

                    self.expect_token(Token::RParen)?;

                    expr = Node::Call {
                        func: Box::new(expr),
                        args,
                    };
                }
                Some(Token::LBracket) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect_token(Token::RBracket)?;

                    expr = Node::Index {
                        expr: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(Token::Dot) => {
                    self.advance();

                    if let Some(Token::Ident(field)) = self.current() {
                        let f = field.clone();
                        self.advance();
                        expr = Node::Field {
                            expr: Box::new(expr),
                            field: f,
                        };
                    } else {
                        return Err("Expected field name after '.'".to_string());
                    }
                }
                Some(Token::As) => {
                    self.advance();
                    let ty = self.parse_type()?;
                    expr = Node::Cast {
                        expr: Box::new(expr),
                        ty: Box::new(ty),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Node, String> {
        match self.current() {
            Some(Token::Number(n)) => {
                let num = n.clone();
                self.advance();
                Ok(Node::Number(num))
            }
            Some(Token::String(s)) => {
                let string = s.clone();
                self.advance();
                Ok(Node::String(string))
            }
            Some(Token::RawString(s)) => {
                let string = s.clone();
                self.advance();
                Ok(Node::RawString(string))
            }
            Some(Token::Ident(name)) => {
                let n = name.clone();
                self.advance();
                Ok(Node::Ident(n))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_token(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LBracket) => {
                self.advance();
                let mut elements = Vec::new();

                while self.current() != Some(&Token::RBracket)
                    && self.current() != Some(&Token::Eof)
                {
                    self.skip_comments();
                    if self.current() == Some(&Token::RBracket) {
                        break;
                    }
                    if !elements.is_empty() {
                        self.expect_token(Token::Comma)?;
                        self.skip_comments();
                        if self.current() == Some(&Token::RBracket) {
                            break;
                        }
                    }
                    elements.push(self.parse_expr()?);
                }

                self.expect_token(Token::RBracket)?;
                Ok(Node::List(elements))
            }
            Some(Token::LBrace) => {
                self.advance();
                let mut fields = Vec::new();

                while self.current() != Some(&Token::RBrace)
                    && self.current() != Some(&Token::Eof)
                {
                    if !fields.is_empty() {
                        self.expect_token(Token::Comma)?;
                    }

                    if let Some(Token::Ident(field_name)) = self.current() {
                        let name = field_name.clone();
                        self.advance();

                        self.expect_token(Token::Colon)?;
                        let value = self.parse_expr()?;
                        fields.push((name, value));
                    } else {
                        break;
                    }
                }

                self.expect_token(Token::RBrace)?;
                Ok(Node::Struct(fields))
            }
            Some(Token::Use) => {
                self.advance();
                if self.match_token(&Token::LParen) {
                    let mut args = Vec::new();
                    while self.current() != Some(&Token::RParen)
                        && self.current() != Some(&Token::Eof)
                    {
                        if !args.is_empty() {
                            self.expect_token(Token::Comma)?;
                        }
                        args.push(self.parse_expr()?);
                    }
                    self.expect_token(Token::RParen)?;
                    Ok(Node::Call {
                        func: Box::new(Node::Ident("use".to_string())),
                        args,
                    })
                } else {
                    Ok(Node::Ident("use".to_string()))
                }
            }
            Some(Token::At) => {
                self.advance();
                let builtin_name = if let Some(Token::Ident(n)) = self.current() {
                    let name = format!("@{}", n);
                    self.advance();
                    name
                } else {
                    return Err("Expected builtin name after @".to_string());
                };

                if self.match_token(&Token::LParen) {
                    let mut args = Vec::new();
                    while self.current() != Some(&Token::RParen)
                        && self.current() != Some(&Token::Eof)
                    {
                        if !args.is_empty() {
                            self.expect_token(Token::Comma)?;
                        }
                        args.push(self.parse_expr()?);
                    }
                    self.expect_token(Token::RParen)?;
                    Ok(Node::Call {
                        func: Box::new(Node::Ident(builtin_name)),
                        args,
                    })
                } else {
                    Ok(Node::Ident(builtin_name))
                }
            }
            _ => Err(format!("Unexpected token in expression: {:?}", self.current())),
        }
    }

    fn get_precedence(&self, token: &Token) -> i32 {
        match token {
            Token::PipePipe => 1,
            Token::AmpAmp => 2,
            Token::Pipe => 3,
            Token::Caret => 4,
            Token::Ampersand => 5,
            Token::EqualEq | Token::NotEq => 6,
            Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => 7,
            Token::LtLt | Token::GtGt => 8,
            Token::Plus | Token::Minus => 9,
            Token::Star | Token::Slash | Token::Percent => 10,
            _ => -1,
        }
    }

    fn token_to_op(&self, token: &Token) -> Result<String, String> {
        match token {
            Token::Plus => Ok("+".to_string()),
            Token::Minus => Ok("-".to_string()),
            Token::Star => Ok("*".to_string()),
            Token::Slash => Ok("/".to_string()),
            Token::Percent => Ok("%".to_string()),
            Token::Equal => Ok("=".to_string()),
            Token::EqualEq => Ok("==".to_string()),
            Token::NotEq => Ok("!=".to_string()),
            Token::Lt => Ok("<".to_string()),
            Token::Gt => Ok(">".to_string()),
            Token::LtEq => Ok("<=".to_string()),
            Token::GtEq => Ok(">=".to_string()),
            Token::AmpAmp => Ok("&&".to_string()),
            Token::PipePipe => Ok("||".to_string()),
            Token::Ampersand => Ok("&".to_string()),
            Token::Pipe => Ok("|".to_string()),
            Token::Caret => Ok("^".to_string()),
            Token::Bang => Ok("!".to_string()),
            Token::Tilde => Ok("~".to_string()),
            Token::Question => Ok("?".to_string()),
            Token::LtLt => Ok("<<".to_string()),
            Token::GtGt => Ok(">>".to_string()),
            _ => Err(format!("Not an operator: {:?}", token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_code(code: &str) -> Result<Vec<Node>, String> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_simple_var_decl() {
        let result = parse_code("val x: i32 = 42;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_decl() {
        let result = parse_code("fn add(a: i32, b: i32) i32 = { return a + b; }");
        assert!(result.is_ok());
    }
}
