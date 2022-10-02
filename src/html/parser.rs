use crate::html::errors::ParseError;
use crate::html::tokenizer::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum NodeKind {
    Tag,
    SoloTag,
    CommentTag,
    DoctypeTag,
    Text,
    Parameters,
    Parameter,
    Identifier,
    String,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub s: String,
    pub params: Option<Box<Node>>,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub children: Option<Vec<Option<Box<Node>>>>,
}

impl Node {
    pub fn new(
        kind: NodeKind,
        params: Option<Box<Node>>,
        lhs: Option<Box<Node>>,
        rhs: Option<Box<Node>>,
        children: Option<Vec<Option<Box<Node>>>>,
        s: String
    ) -> Node {
        return Node {
            kind,
            params,
            lhs,
            rhs,
            children,
            s
        };
    }
}

pub struct Parser {
    token: Option<Box<Token>>,
}

impl Parser {
    pub fn new() -> Parser {
        return Parser { token: None };
    }

    fn current_token(&self) -> Box<Token> {
        return self.token.clone().unwrap();
    }

    fn is_eof(&self) -> bool {
        return self.current_token().kind == TokenKind::Eof;
    }

    fn consume(&mut self) -> Option<Box<Token>> {
        let tok = self.current_token();
        self.token = self.current_token().next;
        return Some(tok);
    }

    fn consume_kind(&mut self, kind: TokenKind) -> Option<Box<Token>> {
        if self.current_token().kind == kind {
            let tok = self.current_token();
            self.token = self.current_token().next;
            return Some(tok);
        }
        return None;
    }

    fn expect_kind(&mut self, kind: TokenKind) -> Result<Option<Box<Token>>, ParseError> {
        if self.current_token().kind == kind {
            let cur = self.current_token();
            self.token = cur.next.clone();
            return Ok(Some(cur));
        }
        return Err(ParseError::UnexpectedToken {
            expected: kind,
            found: *self.current_token(),
        });
    }

    fn expect_text(&mut self, text: String, case_sensitive: bool) -> Result<(), ParseError> {
        return match self.expect_kind(TokenKind::Text) {
            Err(error) => Err(error),
            Ok(tok) => {
                if *&case_sensitive && (tok.clone().unwrap().s == text) {
                    return Ok(());
                }
                if *&!case_sensitive && (tok.clone().unwrap().s.to_lowercase() == text) {
                    return Ok(());
                }
                return Err(ParseError::UnexpectedText {
                    expected: text,
                    found: tok,
                });
            }
        };
    }

    fn parse_text(&mut self) -> Result<Option<Box<Node>>, ParseError> {
        let mut text: String = "".to_string();

        while !self.is_eof() {
            match self.consume_kind(TokenKind::Text) {
                None => break,
                Some(tok) => text += &*tok.s,
            }
        }

        let nd = Node::new(NodeKind::Text, None, None, None, None, text.to_string());
        return Ok(Some(Box::from(nd)));
    }

    fn parse_decl_tag(&mut self) -> Result<Option<Box<Node>>, ParseError> {
        // doctype or comment

        // comment
        if self.consume_kind(TokenKind::Hyphen) != None {
            match self.expect_kind(TokenKind::Hyphen) {
                Err(error) => return Err(error),
                Ok(_) => {
                    let mut comment: String = "".to_string();
                    while !self.is_eof() {
                        if self.consume_kind(TokenKind::Hyphen) != None {
                            if self.consume_kind(TokenKind::Hyphen) != None {
                                if self.consume_kind(TokenKind::TagEnd) != None {
                                    // 終わり
                                    return Ok(Some(Box::from(Node::new(
                                        NodeKind::CommentTag,
                                        None,
                                        None,
                                        None,
                                        None,
                                        comment,
                                    ))));
                                } else {
                                    comment += "--";
                                    continue;
                                }
                            } else {
                                comment += "-";
                                continue;
                            }
                        }

                        if self.consume_kind(TokenKind::Whitespace) != None {
                            comment += " ";
                            continue;
                        }

                        comment += &*self.consume().unwrap().s
                    }
                }
            }
        }

        // consume doctype
        match self.expect_text("doctype".to_string(), false) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
        // consume ws
        match self.expect_kind(TokenKind::Whitespace) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        // type: eg. html
        let doctype_node = match self.expect_kind(TokenKind::Text) {
            Ok(tok) => Node::new(
                NodeKind::DoctypeTag,
                None,
                None,
                None,
                None,
                tok.unwrap().s.to_string().to_lowercase(),
            ),
            Err(err) => return Err(err),
        };

        // consume ">"
        match self.expect_kind(TokenKind::TagEnd) {
            Ok(_) => (),
            Err(err) => return Err(err),
        };

        return Ok(Some(Box::from(doctype_node)));
    }

    fn parse_tag_parameters(&mut self) -> Result<Option<Box<Node>>, ParseError> {
        let mut children: Vec<Option<Box<Node>>> = vec![];

        while !self.is_eof() {
            self.consume_kind(TokenKind::Whitespace);
            // ">" or "/" がきたら中止
            // 最後の処理はtag_bodyに任せるので、consumeしない
            if self.current_token().kind == TokenKind::TagEnd || self.current_token().kind == TokenKind::Slash {
                break;
            }
            // whitespace あるかも
            self.consume_kind(TokenKind::Whitespace);

            // param = value
            // param
            let param_name = match self.consume_kind(TokenKind::Text) {
                Some(tok) => tok,
                None => {
                    return Err(ParseError::UnexpectedToken {
                        expected: TokenKind::Text,
                        found: *self.current_token(),
                    })
                }
            };
            // =
            match self.expect_kind(TokenKind::Assign) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
            // value maybe string
            let value: Token;
            match self.expect_kind(TokenKind::String) {
                Ok(v) => value = *v.unwrap(),
                Err(err) => return Err(err),
            }

            let lhs = Node::new(NodeKind::Identifier, None, None, None, None, param_name.s);
            let rhs = Node::new(NodeKind::String, None, None, None, None, value.s);

            children.push(Some(Box::from(Node::new(
                NodeKind::Parameter,
                Some(Box::from(lhs)),
                Some(Box::from(rhs)),
                None,
                None,
                "".to_string(),
            ))));

            self.consume_kind(TokenKind::Whitespace);
        }

        if children.len() == 0 {
            return Ok(None);
        }

        return Ok(Some(Box::from(Node::new(
            NodeKind::Parameters,
            None,
            None,
            None,
            Some(children),
            "".to_string(),
        ))));
    }

    fn parse_tag(&mut self) -> Result<Option<Box<Node>>, ParseError> {
        if self.consume_kind(TokenKind::Excl) != None {
            return self.parse_decl_tag();
        }

        if self.current_token().kind == TokenKind::Slash {
            return Ok(None);
        }

        let tag_name = match self.expect_kind(TokenKind::Text) {
            Ok(tok) => tok.unwrap().s.to_lowercase(),
            Err(err) => return Err(err),
        };

        // wsが入っている確率が高いので消しておく
        self.consume_kind(TokenKind::Whitespace);

        // parameters
        let params = match self.parse_tag_parameters() {
            Ok(nd) => nd,
            Err(err) => return Err(err),
        };

        // wsが入っている確率が高いので消しておく
        self.consume_kind(TokenKind::Whitespace);

        // Solo tag
        if self.consume_kind(TokenKind::Slash) != None {
            return match self.expect_kind(TokenKind::TagEnd) {
                Ok(_) => Ok(Some(Box::from(Node::new(
                    NodeKind::SoloTag, params, None, None, None, tag_name,
                )))),
                Err(err) => Err(err),
            };
        }

        // ">"
        match self.expect_kind(TokenKind::TagEnd) {
            Ok(_) => {}
            Err(err) => return Err(err),
        }

        let children: Option<Vec<Option<Box<Node>>>> = match self.parse_() {
            Ok(c) => c,
            Err(err) => return Err(err),
        };

        // "/" of close tag
        match self.expect_kind(TokenKind::Slash) {
            Ok(_) => {}
            Err(err) => return Err(err),
        };

        // closing tag name
        let close_tag_name = match self.expect_kind(TokenKind::Text) {
            Ok(tok) => tok.unwrap().s.to_lowercase(),
            Err(err) => return Err(err),
        };

        match self.expect_kind(TokenKind::TagEnd) {
            Ok(_) => {}
            Err(err) => return Err(err),
        }

        // tag miss match: eg. <xxx></yyy>
        if tag_name.clone() != close_tag_name.clone() {
            return Err(ParseError::TagMissMatch {
                open: tag_name,
                close: close_tag_name,
            });
        }

        return Ok(Some(Box::from(Node::new(
            NodeKind::Tag,
            params,
            None,
            None,
            children,
            tag_name.to_string(),
        ))));
    }

    fn parse_(&mut self) -> Result<Option<Vec<Option<Box<Node>>>>, ParseError> {
        let mut nodes: Vec<Option<Box<Node>>> = Vec::new();
        while !self.is_eof() {
            self.consume_kind(TokenKind::Whitespace);
            let nd_result = match self.consume_kind(TokenKind::TagBegin) {
                Some(_) => self.parse_tag(),
                None => self.parse_text(),
            };
            // テキスト、あるいはタグのパースは成功しましたか?
            match nd_result {
                Ok(nd) => match nd {
                    Some(n) => nodes.push(Some(n)),
                    None => break,
                },
                Err(err) => return Err(err),
            }
            self.consume_kind(TokenKind::Whitespace);
        }

        if nodes.len() == 0 {
            return Ok(None);
        }

        return Ok(Some(nodes));
    }

    pub fn parse(
        &mut self,
        token: Option<Box<Token>>,
    ) -> Result<Option<Vec<Option<Box<Node>>>>, ParseError> {
        self.token = Some(token.unwrap());
        match self.parse_() {
            Ok(n) => return Ok(n),
            Err(err) => return Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::html::parser::Parser;
    use crate::html::tokenizer;
    #[test]
    fn parse_only_decl() {
        let mut tokenizer_ = tokenizer::Tokenizer::new("<!doctype html><!-- hello, w--orld -->");
        let tok = tokenizer_.tokenize();

        let mut parser_ = Parser::new();
        let nodes = parser_.parse(tok);
        println!("{:#?}", nodes)
    }

    #[test]
    fn parse_html_tag() {
        let mut tokenizer_ = tokenizer::Tokenizer::new("<html></html>");
        let tok = tokenizer_.tokenize();

        let mut parser_ = Parser::new();
        let nodes = parser_.parse(tok);
        println!("{:#?}", nodes)
    }

    #[test]
    fn parse_html_body() {
        let mut tokenizer_ = tokenizer::Tokenizer::new("<html><body></body></html>");
        let tok = tokenizer_.tokenize();

        let mut parser_ = Parser::new();
        let nodes = parser_.parse(tok);
        println!("{:#?}", nodes)
    }

    #[test]
    fn parse_html_body_h1_img() {
        let html = "<!DOCTYPE html> \
            <html>\
            <body>\
            <h1>hello</h1>\
            <img src=\"https://google.com\"/>\
            </body>\
            </html>";

        let mut tokenizer_ = tokenizer::Tokenizer::new(html);
        let tok = tokenizer_.tokenize();

        let mut parser_ = Parser::new();
        let nodes = parser_.parse(tok);
        println!("{:#?}", nodes)
    }
}