use crate::elab::Error;
use crate::surface::{
    FormalArg, FormalConst, FormalDeclaration, FormalDefault, FormalEntity, FormalField,
    FormalModule, FormalRule, FormalVerb, SurfaceAntecedent, SurfaceDefaultBody, SurfaceEffect,
    SurfaceEvent, SurfaceFieldAccess, SurfaceRecordFieldValue, SurfaceRuleBody,
    SurfaceStateOperation, SurfaceTerm, SurfaceTime, SurfaceVerbKind,
};
use crate::typed::Type;
use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Token {
    Ident(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Colon,
    Subtype,
    Question,
    Dot,
    At,
    Equal,
    FatArrow,
    PlusEqual,
    MinusEqual,
    Leq,
    Ampersand,
}

pub fn parse_formal(source: &str) -> Result<FormalModule, Error> {
    parse_formal_chumsky(source)
}

#[cfg(test)]
pub(crate) fn parse_formal_manual(source: &str) -> Result<FormalModule, Error> {
    ManualParser::new(lex(source)?).parse_module()
}

fn parse_formal_chumsky(source: &str) -> Result<FormalModule, Error> {
    let tokens = lexer()
        .parse(source)
        .map_err(|errors| Error::new(format_chumsky_errors("lex", errors)))?;

    formal_module_parser()
        .parse(tokens)
        .map_err(|errors| Error::new(format_chumsky_errors("parse", errors)))
}

fn format_chumsky_errors<T>(stage: &str, errors: Vec<Simple<T>>) -> String
where
    T: std::fmt::Debug + std::hash::Hash + Eq,
{
    let details = errors
        .into_iter()
        .map(|error| format!("{error:?}"))
        .collect::<Vec<_>>()
        .join("; ");
    format!("formal {stage} error: {details}")
}

fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let ident = text::ident().map(Token::Ident);
    let symbols = choice((
        just("=>").to(Token::FatArrow),
        just("+=").to(Token::PlusEqual),
        just("-=").to(Token::MinusEqual),
        just("<=").to(Token::Leq),
        just("<:").to(Token::Subtype),
        just('≤').to(Token::Leq),
        just('&').to(Token::Ampersand),
        just('{').to(Token::LBrace),
        just('}').to(Token::RBrace),
        just('(').to(Token::LParen),
        just(')').to(Token::RParen),
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('?').to(Token::Question),
        just('.').to(Token::Dot),
        just('@').to(Token::At),
        just('=').to(Token::Equal),
    ));

    choice((ident, symbols))
        .padded()
        .repeated()
        .then_ignore(end())
}

fn formal_module_parser() -> impl Parser<Token, FormalModule, Error = Simple<Token>> {
    let ident = select! { Token::Ident(value) => value };
    let keyword = |expected: &'static str| {
        select! { Token::Ident(value) if value == expected => () }.labelled(expected)
    };

    let ty = recursive(|ty| {
        let entity = ident.clone().map(Type::Entity);
        let set = keyword("Set")
            .ignore_then(ident.clone())
            .map(|name| Type::Set(Box::new(Type::Entity(name))));

        choice((set, entity))
            .then(just(Token::Question).or_not())
            .map(|(ty, optional)| {
                if optional.is_some() {
                    Type::Optional(Box::new(ty))
                } else {
                    ty
                }
            })
            .or(ty.delimited_by(just(Token::LParen), just(Token::RParen)))
    });

    let field = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(ty.clone())
        .map(|(name, ty)| FormalField { name, ty });

    let entity = keyword("entity")
        .ignore_then(ident.clone())
        .then(
            just(Token::Subtype)
                .ignore_then(ident.clone())
                .or_not()
                .map(|supertype| supertype.into_iter().collect()),
        )
        .then(
            field
                .repeated()
                .delimited_by(just(Token::LBrace), just(Token::RBrace))
                .or_not()
                .map(Option::unwrap_or_default),
        )
        .map(|((name, supertypes), fields)| {
            FormalDeclaration::Entity(FormalEntity {
                name,
                supertypes,
                fields,
            })
        });

    let const_decl = keyword("const")
        .ignore_then(ident.clone())
        .then_ignore(just(Token::Colon))
        .then(ty.clone())
        .map(|(name, ty)| FormalDeclaration::Const(FormalConst { name, ty }));

    let arg = ident
        .clone()
        .then_ignore(just(Token::Colon))
        .then(ty.clone())
        .map(|(name, ty)| FormalArg { name, ty });

    let args = arg
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .delimited_by(just(Token::LParen), just(Token::RParen));

    let term = recursive(|term| {
        let record_field = ident
            .clone()
            .then_ignore(just(Token::Equal))
            .then(term.clone())
            .map(|(field, value)| SurfaceRecordFieldValue { field, value });

        ident
            .clone()
            .then(
                record_field
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace))
                    .or_not(),
            )
            .map(|(name, fields)| match fields {
                Some(fields) => SurfaceTerm::RecordLiteral {
                    entity: name,
                    fields,
                },
                None if name == "none" => SurfaceTerm::None,
                None => SurfaceTerm::Var(name),
            })
    });

    let field_access = ident
        .clone()
        .then(
            just(Token::Dot)
                .ignore_then(ident.clone())
                .repeated()
                .at_least(1),
        )
        .map(|(base, path)| SurfaceFieldAccess { base, path });

    let state_operation = field_access
        .then(choice((
            just(Token::PlusEqual).to(true),
            just(Token::MinusEqual).to(false),
        )))
        .then(term.clone())
        .map(|((target, add), value)| {
            if add {
                SurfaceStateOperation::AddToSet { target, value }
            } else {
                SurfaceStateOperation::RemoveFromSet { target, value }
            }
        });

    let effect_body = state_operation
        .repeated()
        .map(|operations| SurfaceEffect { operations });

    let verb = choice((
        keyword("seeded").to(SurfaceVerbKind::Seeded),
        keyword("effect").to(SurfaceVerbKind::Effect),
    ))
    .then_ignore(keyword("verb"))
    .then(ident.clone())
    .then(args.clone())
    .then(just(Token::FatArrow).ignore_then(effect_body).or_not())
    .map(|(((kind, name), args), effect)| {
        FormalDeclaration::Verb(FormalVerb {
            name,
            kind,
            args,
            effect,
        })
    });

    let time = just(Token::At)
        .ignore_then(choice((
            keyword("after")
                .ignore_then(ident.clone())
                .map(SurfaceTime::After),
            ident.clone().map(SurfaceTime::At),
        )))
        .or_not();

    let event = ident
        .clone()
        .then(
            ident
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(time)
        .map(|((verb, args), time)| SurfaceEvent { verb, args, time });

    let antecedent = event
        .clone()
        .map(SurfaceAntecedent::Event)
        .then(
            just(Token::Ampersand)
                .ignore_then(event.clone().map(SurfaceAntecedent::Event))
                .repeated(),
        )
        .map(|(first, rest)| {
            rest.into_iter().fold(first, |left, right| {
                SurfaceAntecedent::And(Box::new(left), Box::new(right))
            })
        });

    let rule_application = keyword("rule")
        .ignore_then(ident.clone())
        .then_ignore(just(Token::Equal))
        .then(ident.clone())
        .then(
            ident
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map(|((name, rule), args)| {
            FormalDeclaration::Rule(FormalRule {
                name,
                body: SurfaceRuleBody::Application { rule, args },
            })
        });

    let horn_rule = keyword("rule")
        .ignore_then(ident.clone())
        .then(
            ident
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then_ignore(just(Token::Equal))
        .then(antecedent)
        .then_ignore(just(Token::Leq))
        .then(event.clone())
        .map(|(((name, params), premise), conclusion)| {
            FormalDeclaration::Rule(FormalRule {
                name,
                body: SurfaceRuleBody::Horn {
                    params,
                    premise,
                    conclusion,
                },
            })
        });

    let rule = choice((horn_rule, rule_application));

    let default = keyword("default")
        .ignore_then(ident.clone())
        .then_ignore(just(Token::Equal))
        .then_ignore(keyword("supernormal"))
        .then(ident.clone())
        .map(|(name, rule)| {
            FormalDeclaration::Default(FormalDefault {
                name,
                body: SurfaceDefaultBody::Supernormal { rule },
            })
        });

    let declaration = choice((entity, const_decl, verb, rule, default));

    keyword("module")
        .ignore_then(ident.clone())
        .then(declaration.repeated())
        .then_ignore(end())
        .map(|(name, declarations)| FormalModule { name, declarations })
}

#[cfg(test)]
fn lex(source: &str) -> Result<Vec<Token>, Error> {
    let mut chars = source.char_indices().peekable();
    let mut tokens = Vec::new();

    while let Some((idx, ch)) = chars.next() {
        match ch {
            c if c.is_whitespace() => {}
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
            '?' => tokens.push(Token::Question),
            '<' => {
                if matches!(chars.peek(), Some((_, ':'))) {
                    chars.next();
                    tokens.push(Token::Subtype);
                } else if matches!(chars.peek(), Some((_, '='))) {
                    chars.next();
                    tokens.push(Token::Leq);
                } else {
                    return Err(Error::new(format!("unexpected `<` at byte {idx}")));
                }
            }
            '.' => tokens.push(Token::Dot),
            '@' => tokens.push(Token::At),
            '≤' => tokens.push(Token::Leq),
            '&' => tokens.push(Token::Ampersand),
            '=' => {
                if matches!(chars.peek(), Some((_, '>'))) {
                    chars.next();
                    tokens.push(Token::FatArrow);
                } else {
                    tokens.push(Token::Equal);
                }
            }
            '+' => {
                if matches!(chars.peek(), Some((_, '='))) {
                    chars.next();
                    tokens.push(Token::PlusEqual);
                } else {
                    return Err(Error::new(format!("unexpected `+` at byte {idx}")));
                }
            }
            '-' => {
                if matches!(chars.peek(), Some((_, '='))) {
                    chars.next();
                    tokens.push(Token::MinusEqual);
                } else {
                    return Err(Error::new(format!("unexpected `-` at byte {idx}")));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = idx;
                let mut end = idx + ch.len_utf8();
                while let Some((next_idx, next)) = chars.peek().copied() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        chars.next();
                        end = next_idx + next.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(source[start..end].to_string()));
            }
            other => return Err(Error::new(format!("unexpected `{other}` at byte {idx}"))),
        }
    }

    Ok(tokens)
}

#[cfg(test)]
struct ManualParser {
    tokens: Vec<Token>,
    pos: usize,
}

#[cfg(test)]
impl ManualParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_module(&mut self) -> Result<FormalModule, Error> {
        self.expect_ident_value("module")?;
        let name = self.expect_ident()?;
        let mut declarations = Vec::new();

        while !self.is_eof() {
            let head = self.peek_ident()?;
            match head {
                "entity" => declarations.push(FormalDeclaration::Entity(self.parse_entity()?)),
                "const" => declarations.push(FormalDeclaration::Const(self.parse_const()?)),
                "seeded" | "effect" => {
                    declarations.push(FormalDeclaration::Verb(self.parse_verb()?))
                }
                "rule" => declarations.push(FormalDeclaration::Rule(self.parse_rule()?)),
                "default" => declarations.push(FormalDeclaration::Default(self.parse_default()?)),
                other => return Err(Error::new(format!("unexpected declaration `{other}`"))),
            }
        }

        Ok(FormalModule { name, declarations })
    }

    fn parse_entity(&mut self) -> Result<FormalEntity, Error> {
        self.expect_ident_value("entity")?;
        let name = self.expect_ident()?;
        let mut supertypes = Vec::new();
        if self.eat(&Token::Subtype) {
            supertypes.push(self.expect_ident()?);
        }
        let mut fields = Vec::new();

        if self.eat(&Token::LBrace) {
            while !self.eat(&Token::RBrace) {
                let field_name = self.expect_ident()?;
                self.expect(&Token::Colon)?;
                let ty = self.parse_type()?;
                fields.push(FormalField {
                    name: field_name,
                    ty,
                });
            }
        }

        Ok(FormalEntity {
            name,
            supertypes,
            fields,
        })
    }

    fn parse_const(&mut self) -> Result<FormalConst, Error> {
        self.expect_ident_value("const")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        Ok(FormalConst { name, ty })
    }

    fn parse_verb(&mut self) -> Result<FormalVerb, Error> {
        let kind = match self.expect_ident()?.as_str() {
            "seeded" => SurfaceVerbKind::Seeded,
            "effect" => SurfaceVerbKind::Effect,
            other => return Err(Error::new(format!("expected verb kind, got `{other}`"))),
        };
        self.expect_ident_value("verb")?;
        let name = self.expect_ident()?;
        let args = self.parse_args()?;
        let effect = if matches!(kind, SurfaceVerbKind::Effect) {
            if self.eat(&Token::FatArrow) {
                Some(self.parse_effect()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(FormalVerb {
            name,
            kind,
            args,
            effect,
        })
    }

    fn parse_args(&mut self) -> Result<Vec<FormalArg>, Error> {
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        while !self.eat(&Token::RParen) {
            let name = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            args.push(FormalArg { name, ty });
            self.eat(&Token::Comma);
        }
        Ok(args)
    }

    fn parse_type(&mut self) -> Result<Type, Error> {
        let name = self.expect_ident()?;
        let ty = if name == "Set" {
            Type::Set(Box::new(Type::Entity(self.expect_ident()?)))
        } else {
            Type::Entity(name)
        };

        if self.eat(&Token::Question) {
            Ok(Type::Optional(Box::new(ty)))
        } else {
            Ok(ty)
        }
    }

    fn parse_effect(&mut self) -> Result<SurfaceEffect, Error> {
        let mut operations = Vec::new();

        while self.starts_field_access() {
            let base = self.expect_ident()?;
            let mut path = Vec::new();
            while self.eat(&Token::Dot) {
                path.push(self.expect_ident()?);
            }
            let target = SurfaceFieldAccess { base, path };

            if self.eat(&Token::PlusEqual) {
                operations.push(SurfaceStateOperation::AddToSet {
                    target,
                    value: self.parse_term()?,
                });
            } else if self.eat(&Token::MinusEqual) {
                operations.push(SurfaceStateOperation::RemoveFromSet {
                    target,
                    value: self.parse_term()?,
                });
            } else {
                return Err(Error::new("expected `+=` or `-=` in effect body"));
            }
        }

        Ok(SurfaceEffect { operations })
    }

    fn parse_term(&mut self) -> Result<SurfaceTerm, Error> {
        let name = self.expect_ident()?;
        if name == "none" {
            return Ok(SurfaceTerm::None);
        }

        if self.eat(&Token::LBrace) {
            let mut fields = Vec::new();
            while !self.eat(&Token::RBrace) {
                let field = self.expect_ident()?;
                self.expect(&Token::Equal)?;
                let value = self.parse_term()?;
                fields.push(SurfaceRecordFieldValue { field, value });
                self.eat(&Token::Comma);
            }
            Ok(SurfaceTerm::RecordLiteral {
                entity: name,
                fields,
            })
        } else {
            Ok(SurfaceTerm::Var(name))
        }
    }

    fn parse_rule(&mut self) -> Result<FormalRule, Error> {
        self.expect_ident_value("rule")?;
        let name = self.expect_ident()?;
        if self.eat(&Token::LParen) {
            let mut params = Vec::new();
            while !self.eat(&Token::RParen) {
                params.push(self.expect_ident()?);
                self.eat(&Token::Comma);
            }
            self.expect(&Token::Equal)?;
            let premise = self.parse_antecedent()?;
            self.expect(&Token::Leq)?;
            let conclusion = self.parse_event()?;

            Ok(FormalRule {
                name,
                body: SurfaceRuleBody::Horn {
                    params,
                    premise,
                    conclusion,
                },
            })
        } else {
            self.expect(&Token::Equal)?;
            let rule = self.expect_ident()?;
            self.expect(&Token::LParen)?;
            let mut args = Vec::new();
            while !self.eat(&Token::RParen) {
                args.push(self.expect_ident()?);
                self.eat(&Token::Comma);
            }
            Ok(FormalRule {
                name,
                body: SurfaceRuleBody::Application { rule, args },
            })
        }
    }

    fn parse_antecedent(&mut self) -> Result<SurfaceAntecedent, Error> {
        let mut antecedent = SurfaceAntecedent::Event(self.parse_event()?);
        while self.eat(&Token::Ampersand) {
            let right = SurfaceAntecedent::Event(self.parse_event()?);
            antecedent = SurfaceAntecedent::And(Box::new(antecedent), Box::new(right));
        }
        Ok(antecedent)
    }

    fn parse_default(&mut self) -> Result<FormalDefault, Error> {
        self.expect_ident_value("default")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Equal)?;
        self.expect_ident_value("supernormal")?;
        let rule = self.expect_ident()?;

        Ok(FormalDefault {
            name,
            body: SurfaceDefaultBody::Supernormal { rule },
        })
    }

    fn parse_event(&mut self) -> Result<SurfaceEvent, Error> {
        let verb = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        while !self.eat(&Token::RParen) {
            args.push(self.expect_ident()?);
            self.eat(&Token::Comma);
        }
        let time = if self.eat(&Token::At) {
            Some(if self.eat_ident("after") {
                SurfaceTime::After(self.expect_ident()?)
            } else {
                SurfaceTime::At(self.expect_ident()?)
            })
        } else {
            None
        };

        Ok(SurfaceEvent { verb, args, time })
    }

    fn expect_ident_value(&mut self, expected: &str) -> Result<(), Error> {
        let got = self.expect_ident()?;
        if got == expected {
            Ok(())
        } else {
            Err(Error::new(format!("expected `{expected}`, got `{got}`")))
        }
    }

    fn expect_ident(&mut self) -> Result<String, Error> {
        match self.tokens.get(self.pos) {
            Some(Token::Ident(value)) => {
                self.pos += 1;
                Ok(value.clone())
            }
            got => Err(Error::new(format!("expected identifier, got {got:?}"))),
        }
    }

    fn peek_ident(&self) -> Result<&str, Error> {
        match self.tokens.get(self.pos) {
            Some(Token::Ident(value)) => Ok(value),
            got => Err(Error::new(format!("expected identifier, got {got:?}"))),
        }
    }

    fn eat_ident(&mut self, expected: &str) -> bool {
        if matches!(self.tokens.get(self.pos), Some(Token::Ident(value)) if value == expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn starts_field_access(&self) -> bool {
        matches!(self.tokens.get(self.pos), Some(Token::Ident(_)))
            && matches!(self.tokens.get(self.pos + 1), Some(Token::Dot))
    }

    fn expect(&mut self, expected: &Token) -> Result<(), Error> {
        if self.eat(expected) {
            Ok(())
        } else {
            Err(Error::new(format!(
                "expected {expected:?}, got {:?}",
                self.tokens.get(self.pos)
            )))
        }
    }

    fn eat(&mut self, expected: &Token) -> bool {
        if self.tokens.get(self.pos) == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn is_eof(&self) -> bool {
        self.pos == self.tokens.len()
    }
}
