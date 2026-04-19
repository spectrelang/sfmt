use crate::parser::Node;

const INDENT: &str = "    "; // 4 spaces

fn node_kind(node: &Node) -> &'static str {
    match node {
        Node::FnDecl { .. } => "fn",
        Node::TypeDecl { .. } | Node::UnionDecl { .. } | Node::EnumDecl { .. } => "type",
        Node::Comment(_) => "comment",
        Node::VarDecl { value: Some(v), .. } => {
            if let Node::Call { func, .. } = v.as_ref() {
                if let Node::Ident(n) = func.as_ref() {
                    if n == "use" {
                        return "use-val";
                    }
                }
            }
            "val"
        }
        _ => "val",
    }
}

fn emit_aligned_comment_pairs(output: &mut String, pairs: &[(String, String)]) {
    let max_len = pairs.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    for (line, comment) in pairs {
        output.push_str(line);
        let padding = max_len - line.len();
        for _ in 0..padding {
            output.push(' ');
        }
        output.push_str(" //");
        output.push_str(comment);
        output.push('\n');
    }
}

fn spacing_between(prev: &str, cur: &str) -> usize {
    match (prev, cur) {
        ("fn", "fn") => 1,
        ("fn", _) | (_, "fn") => 2,
        ("type", k) | (k, "type") if k != "type" => 1,
        ("use-val", "val") | ("val", "use-val") => 1,
        _ => 0,
    }
}

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
        let mut prev_kind = "";
        let mut i = 0;

        while i < nodes.len() {
            let cur_kind = node_kind(&nodes[i]);

            if cur_kind == "comment" {
                let next_real_kind = nodes[i..].iter()
                    .find(|n| node_kind(n) != "comment")
                    .map(node_kind)
                    .unwrap_or("");

                if !prev_kind.is_empty() && !next_real_kind.is_empty() {
                    let blank_lines = spacing_between(prev_kind, next_real_kind);
                    for _ in 0..blank_lines {
                        self.output.push('\n');
                    }
                }

                while i < nodes.len() && node_kind(&nodes[i]) == "comment" {
                    self.format_node(&nodes[i]);
                    i += 1;
                }

                prev_kind = "comment";
                continue;
            }

            // Check if this node is immediately followed by a comment (inline comment)
            if i + 1 < nodes.len() && node_kind(&nodes[i + 1]) == "comment" {
                let line = self.format_to_string(&nodes[i]);
                if !line.contains('\n') {
                    // Collect consecutive (node, comment) pairs into a group
                    let mut pairs: Vec<(String, String)> = Vec::new();
                    let mut j = i;
                    loop {
                        let l = if j == i {
                            line.clone()
                        } else {
                            let s = self.format_to_string(&nodes[j]);
                            if s.contains('\n') { break; }
                            s
                        };
                        let c = if let Node::Comment(c) = &nodes[j + 1] { c.clone() } else { break };
                        pairs.push((l, c));
                        j += 2;
                        if j >= nodes.len() || node_kind(&nodes[j]) == "comment" || j + 1 >= nodes.len() || node_kind(&nodes[j + 1]) != "comment" {
                            break;
                        }
                    }

                    if !prev_kind.is_empty() && prev_kind != "comment" {
                        let blank_lines = spacing_between(prev_kind, cur_kind);
                        for _ in 0..blank_lines { self.output.push('\n'); }
                    }

                    emit_aligned_comment_pairs(&mut self.output, &pairs);

                    prev_kind = cur_kind;
                    i = j;
                    continue;
                }
            }

            if !prev_kind.is_empty() && prev_kind != "comment" {
                let blank_lines = spacing_between(prev_kind, cur_kind);
                for _ in 0..blank_lines {
                    self.output.push('\n');
                }
            }

            self.format_node(&nodes[i]);
            prev_kind = cur_kind;
            i += 1;
        }

        self.output.trim_end().to_string()
    }

    fn format_to_string(&mut self, node: &Node) -> String {
        let saved = std::mem::take(&mut self.output);
        self.format_node(node);
        let mut result = std::mem::take(&mut self.output);
        self.output = saved;
        if result.ends_with('\n') {
            result.pop();
        }
        result
    }

    fn format_stmts_aligned(&mut self, nodes: &[Node]) {
        let mut i = 0;
        while i < nodes.len() {
            if i + 1 < nodes.len()
                && node_kind(&nodes[i]) != "comment"
                && node_kind(&nodes[i + 1]) == "comment"
            {
                let line = self.format_to_string(&nodes[i]);
                if !line.contains('\n') {
                    let mut pairs: Vec<(String, String)> = Vec::new();
                    let mut j = i;
                    loop {
                        let l = if j == i {
                            line.clone()
                        } else {
                            let s = self.format_to_string(&nodes[j]);
                            if s.contains('\n') { break; }
                            s
                        };
                        let c = if let Node::Comment(c) = &nodes[j + 1] { c.clone() } else { break };
                        pairs.push((l, c));
                        j += 2;
                        if j >= nodes.len() || node_kind(&nodes[j]) == "comment" || j + 1 >= nodes.len() || node_kind(&nodes[j + 1]) != "comment" {
                            break;
                        }
                    }
                    emit_aligned_comment_pairs(&mut self.output, &pairs);
                    i = j;
                    continue;
                }
            }
            self.format_node(&nodes[i]);
            i += 1;
        }
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

                let max_key_len = fields.iter().map(|(k, _, _)| k.len()).max().unwrap_or(0);

                // Build each field line (without comment) to find max width for comment alignment
                let indent_str = self.current_indent();
                let field_lines: Vec<(String, Option<String>)> = fields.iter().enumerate().map(|(i, (key, ty, comment))| {
                    let padded_key = format!("{:<width$}", key, width = max_key_len);
                    let type_str = self.node_to_string(ty);
                    let mut line = format!("{}{}: {}", indent_str, padded_key, type_str);
                    if i < fields.len() - 1 {
                        line.push(',');
                    }
                    (line, comment.clone())
                }).collect();

                let max_line_len = field_lines.iter()
                    .filter(|(_, c)| c.is_some())
                    .map(|(l, _)| l.len())
                    .max()
                    .unwrap_or(0);

                for (line, comment) in &field_lines {
                    self.output.push_str(line);
                    if let Some(c) = comment {
                        let padding = max_line_len - line.len();
                        for _ in 0..padding { self.output.push(' '); }
                        self.output.push_str("  //");
                        self.output.push_str(c);
                    }
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
                self.format_stmts_aligned(statements);
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
                self.output.push('\n');
            }
            Node::Break => {
                self.output.push_str(&self.push_indent("break\n"));
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
                self.output.push('\n');
            }
            Node::Comment(c) => {
                self.output.push_str(&self.push_indent("//"));
                self.output.push_str(c);
                self.output.push('\n');
            }
            _ => {
                self.output.push_str(&self.push_indent(""));
                self.output.push_str(&self.node_to_string(node));
                self.output.push('\n');
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
