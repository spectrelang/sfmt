use crate::parser::Node;

const INDENT: &str = "    "; // 4 spaces

pub struct Formatter {
    indent_level: usize,
    output: String,
    in_type_decl: bool,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            indent_level: 0,
            output: String::new(),
            in_type_decl: false,
        }
    }

    fn current_indent(&self) -> String {
        INDENT.repeat(self.indent_level)
    }

    fn push_indent(&self, text: &str) -> String {
        format!("{}{}", self.current_indent(), text)
    }

    pub fn format(&mut self, nodes: &[Node]) -> String {
        let mut prev_was_fn = false;
        for (i, node) in nodes.iter().enumerate() {
            let is_fn = matches!(node, Node::FnDecl { .. });

            if i > 0 {
                if is_fn && prev_was_fn {
                    self.output.push('\n');
                } else if is_fn || prev_was_fn {
                    self.output.push('\n');
                    self.output.push('\n');
                }
            }

            self.format_node(node);
            prev_was_fn = is_fn;
        }
        self.output.trim_end().to_string()
    }

    fn format_node(&mut self, node: &Node) {
        match node {
            Node::VarDecl {
                is_pub,
                is_mut,
                name,
                ty,
                value,
            } => {
                let mut line = self.current_indent();
                if *is_pub {
                    line.push_str("pub ");
                }
                line.push_str("val ");
                if *is_mut {
                    line.push_str("mut ");
                }
                line.push_str(name);

                if let Some(ty_node) = ty {
                    line.push_str(": ");
                    line.push_str(&self.node_to_string(ty_node));
                }

                if let Some(val_node) = value {
                    line.push_str(" = ");
                    if let Node::List(elements) = val_node.as_ref() {
                        self.output.push_str(&line);
                        self.output.push_str("[\n");
                        for (i, elem) in elements.iter().enumerate() {
                            self.output.push_str(&self.push_indent("    "));
                            self.output.push_str(&self.node_to_string(elem));
                            if i < elements.len() - 1 {
                                self.output.push(',');
                            }
                            self.output.push('\n');
                        }
                        self.output.push_str(&self.push_indent("]\n"));
                        return;
                    } else {
                        line.push_str(&self.node_to_string(val_node));
                    }
                }

                line.push('\n');
                self.output.push_str(&line);
            }
            Node::FnDecl {
                is_pub,
                receiver,
                name,
                params,
                return_type,
                body,
            } => {
                let mut line = self.current_indent();
                if *is_pub {
                    line.push_str("pub ");
                }
                line.push_str("fn ");

                if let Some(recv) = receiver {
                    line.push('(');
                    line.push_str(&self.node_to_string(recv));
                    line.push_str(") ");
                }

                line.push_str(name);
                line.push('(');

                for (i, (param_name, param_type)) in params.iter().enumerate() {
                    if i > 0 {
                        line.push_str(", ");
                    }
                    line.push_str(param_name);
                    line.push_str(": ");
                    line.push_str(&self.node_to_string(param_type));
                }

                line.push(')');

                if let Some(rt) = return_type {
                    line.push(' ');
                    line.push_str(&self.node_to_string(rt));
                }

                self.output.push_str(&line);

                if let Some(body_node) = body {
                    self.output.push_str(" = ");
                    self.format_node(body_node);
                } else {
                    self.output.push('\n');
                }
            }
            Node::TypeDecl {
                is_pub,
                name,
                fields,
            } => {
                let mut line = self.current_indent();
                if *is_pub {
                    line.push_str("pub ");
                }
                line.push_str("type ");
                line.push_str(name);
                line.push_str(" = {\n");
                self.output.push_str(&line);

                self.indent_level += 1;

                let max_key_len = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

                for (key, ty) in fields {
                    let padded_key = format!("{:<width$}", key, width = max_key_len);
                    let type_str = self.node_to_string(ty);
                    self.output.push_str(&self.push_indent(""));
                    self.output.push_str(&padded_key);
                    self.output.push_str(": ");
                    self.output.push_str(&type_str);
                    self.output.push('\n');
                }

                self.indent_level -= 1;
                self.output.push_str(&self.push_indent("}\n"));
            }
            Node::UnionDecl {
                is_pub,
                name,
                variants,
            } => {
                let mut line = self.current_indent();
                if *is_pub {
                    line.push_str("pub ");
                }
                line.push_str("union ");
                line.push_str(name);
                line.push_str(" = {\n");
                self.output.push_str(&line);

                self.indent_level += 1;

                for (i, (variant_name, types)) in variants.iter().enumerate() {
                    if i > 0 {
                        self.output.pop();
                        self.output.push_str(" | ");
                    }

                    self.output.push_str(&self.push_indent(""));
                    self.output.push_str(variant_name);

                    if !types.is_empty() {
                        self.output.push('(');
                        for (j, ty) in types.iter().enumerate() {
                            if j > 0 {
                                self.output.push_str(", ");
                            }
                            self.output.push_str(&self.node_to_string(ty));
                        }
                        self.output.push(')');
                    }

                    self.output.push('\n');
                }

                self.indent_level -= 1;
                self.output.push_str(&self.push_indent("}\n"));
            }
            Node::EnumDecl {
                is_pub,
                name,
                variants,
            } => {
                let mut line = self.current_indent();
                if *is_pub {
                    line.push_str("pub ");
                }
                line.push_str("enum ");
                line.push_str(name);
                line.push_str(" = {\n");
                self.output.push_str(&line);

                self.indent_level += 1;

                for (i, variant) in variants.iter().enumerate() {
                    if i > 0 {
                        self.output.pop();
                        self.output.push_str(", ");
                    } else {
                        self.output.push_str(&self.push_indent(""));
                    }

                    self.output.push_str(variant);

                    if i == variants.len() - 1 {
                        self.output.push('\n');
                    }
                }

                self.indent_level -= 1;
                self.output.push_str(&self.push_indent("}\n"));
            }
            Node::TestBlock(body) => {
                self.output.push_str(&self.push_indent("test "));
                self.format_node(body);
            }
            Node::Block(statements) => {
                self.output.push_str("{\n");
                self.indent_level += 1;

                for stmt in statements {
                    self.format_node(stmt);
                }

                self.indent_level -= 1;
                self.output.push_str(&self.push_indent("}\n"));
            }
            Node::If {
                condition,
                then_body,
                elif_parts,
                else_body,
            } => {
                self.output.push_str(&self.push_indent("if "));
                self.output.push_str(&self.node_to_string(condition));
                self.output.push(' ');
                self.format_node(then_body);

                for (elif_cond, elif_body) in elif_parts {
                    if self.output.ends_with('\n') {
                        self.output.pop();
                    }
                    self.output.push_str(" elif ");
                    self.output.push_str(&self.node_to_string(elif_cond));
                    self.output.push(' ');
                    self.format_node(elif_body);
                }

                if let Some(else_bl) = else_body {
                    if self.output.ends_with('\n') {
                        self.output.pop();
                    }
                    self.output.push_str(" else ");
                    self.format_node(else_bl);
                }
            }
            Node::For {
                pattern,
                iterable,
                body,
            } => {
                self.output.push_str(&self.push_indent("for"));

                if let Some(pat) = pattern {
                    self.output.push(' ');
                    self.output.push_str(pat);

                    if let Some(iter) = iterable {
                        self.output.push_str(" in ");
                        self.output.push_str(&self.node_to_string(iter));
                    }
                }

                self.output.push(' ');
                self.format_node(body);
            }
            Node::Return(expr) => {
                self.output.push_str(&self.push_indent("return"));
                if let Some(e) = expr {
                    self.output.push(' ');
                    self.output.push_str(&self.node_to_string(e));
                }
                self.output.push_str(";\n");
            }
            Node::Break => {
                self.output.push_str(&self.push_indent("break;\n"));
            }
            Node::Match { expr, arms } => {
                self.output.push_str(&self.push_indent("match "));
                self.output.push_str(&self.node_to_string(expr));
                self.output.push_str(" {\n");

                self.indent_level += 1;

                for (pattern, _bindings, body) in arms {
                    self.output.push_str(&self.push_indent(""));
                    self.output.push_str(pattern);
                    self.output.push_str(" => ");
                    self.format_node(body);
                }

                self.indent_level -= 1;
                self.output.push_str(&self.push_indent("}\n"));
            }
            Node::Assert(expr) => {
                self.output.push_str(&self.push_indent("assert "));
                self.output.push_str(&self.node_to_string(expr));
                self.output.push_str(";\n");
            }
            Node::Comment(c) => {
                self.output.push_str(&self.push_indent("// "));
                self.output.push_str(c);
                self.output.push('\n');
            }
            _ => {
                self.output.push_str(&self.push_indent(""));
                self.output.push_str(&self.node_to_string(node));
                self.output.push_str(";\n");
            }
        }
    }

    fn node_to_string(&self, node: &Node) -> String {
        match node {
            Node::Ident(name) => name.clone(),
            Node::Number(n) => n.clone(),
            Node::String(s) => format!("\"{}\"", s),
            Node::RawString(s) => format!("\\\\ {}", s),
            Node::Binary { op, left, right } => {
                format!(
                    "{} {} {}",
                    self.node_to_string(left),
                    op,
                    self.node_to_string(right)
                )
            }
            Node::Unary { op, expr } => {
                format!("{}{}", op, self.node_to_string(expr))
            }
            Node::Call { func, args } => {
                let mut result = self.node_to_string(func);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.node_to_string(arg));
                }
                result.push(')');
                result
            }
            Node::Index { expr, index } => {
                format!(
                    "{}[{}]",
                    self.node_to_string(expr),
                    self.node_to_string(index)
                )
            }
            Node::Field { expr, field } => {
                format!("{}.{}", self.node_to_string(expr), field)
            }
            Node::Cast { expr, ty } => {
                format!("{} as {}", self.node_to_string(expr), self.node_to_string(ty))
            }
            Node::List(elements) => {
                let mut result = String::from("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.node_to_string(elem));
                }
                result.push(']');
                result
            }
            Node::Struct(fields) => {
                let mut result = String::from("{");
                for (i, (key, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(key);
                    result.push_str(": ");
                    result.push_str(&self.node_to_string(val));
                }
                result.push('}');
                result
            }
            _ => format!("{:?}", node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_var_decl() {
        let node = Node::VarDecl {
            is_pub: false,
            is_mut: false,
            name: "x".to_string(),
            ty: Some(Box::new(Node::Ident("i32".to_string()))),
            value: Some(Box::new(Node::Number("42".to_string()))),
        };

        let mut formatter = Formatter::new();
        formatter.format_node(&node);

        assert!(formatter.output.contains("val x: i32 = 42"));
    }
}
