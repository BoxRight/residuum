use crate::elab::Error;
use crate::surface::{
    FormalArg, FormalConst, FormalDeclaration, FormalDefault, FormalEntity, FormalExperiment,
    FormalField, FormalModule, FormalRule, FormalVerb, SurfaceAntecedent, SurfaceDefaultBody,
    SurfaceEffect, SurfaceEvent, SurfaceFieldAccess, SurfaceRecordFieldValue, SurfaceRuleBody,
    SurfaceStateOperation, SurfaceTerm, SurfaceTime, SurfaceVerbKind,
};
use crate::typed::{ExperimentInputKind, ExperimentQueryKind, Polarity, Type};
use chumsky::prelude::*;

#[cfg(test)]
mod comment_tests;

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
    Multimap,
    Ampersand,
    Tilde,
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
    let line_comment = just("//")
        .or(just("#"))
        .ignore_then(filter(|c: &char| *c != '\n' && *c != '\r').repeated())
        .ignored();
    let block_comment = just("/*").ignore_then(take_until(just("*/"))).ignored();
    // Each repeated trivia item consumes a character or a comment opener.
    // The trivia list may be empty; repeated tokens still consume input.
    let trivia = choice((
        line_comment,
        block_comment,
        filter(|c: &char| c.is_whitespace()).ignored(),
    ))
    .repeated()
    .ignored();
    let ident = text::ident().map(Token::Ident);
    let symbols = choice((
        just("=>").to(Token::FatArrow),
        just("+=").to(Token::PlusEqual),
        just("-=").to(Token::MinusEqual),
        just("-o").to(Token::Multimap),
        just("<=").to(Token::Leq),
        just("<:").to(Token::Subtype),
        just('≤').to(Token::Leq),
        just('⊸').to(Token::Multimap),
        just('&').to(Token::Ampersand),
        just('~').to(Token::Tilde),
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

    // A repeated token always consumes its identifier/symbol before trivia.
    trivia
        .clone()
        .ignore_then(choice((ident, symbols)).then_ignore(trivia).repeated())
        .then_ignore(end())
}

fn formal_module_parser() -> impl Parser<Token, FormalModule, Error = Simple<Token>> {
    let ident = select! { Token::Ident(value) => value };
    let keyword = |expected: &'static str| {
        select! { Token::Ident(value) if value == expected => () }.labelled(expected)
    };

    let ty = recursive(|ty| {
        let entity = choice((
            keyword("PropositionTime").to(Type::PropositionTime),
            ident.clone().map(Type::Entity),
        ));
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
        keyword("derived").to(SurfaceVerbKind::Derived),
        keyword("effect").to(SurfaceVerbKind::Effect),
    ))
    .then_ignore(keyword("verb"))
    .then(ident.clone())
    .then(args.clone())
    .then(just(Token::FatArrow).ignore_then(effect_body).or_not())
    .then(
        just(Token::Equal)
            .ignore_then(keyword("program"))
            .ignore_then(ident.clone())
            .or_not(),
    )
    .map(|((((kind, name), args), effect), program)| {
        FormalDeclaration::Verb(FormalVerb {
            name,
            kind,
            args,
            effect,
            program,
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

    let event = just(Token::Tilde)
        .or_not()
        .then(ident.clone())
        .then(
            ident
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .then(time)
        .map(|(((negative, verb), args), time)| SurfaceEvent {
            polarity: if negative.is_some() {
                Polarity::EvidentialNot
            } else {
                Polarity::Positive
            },
            verb,
            args,
            time,
        });

    // I consumes a token and produces a neutral witness independently of Known.
    // Trying the event first preserves I(...) as an ordinary verb application.
    let antecedent_atom = choice((
        event.clone().map(SurfaceAntecedent::Event),
        keyword("I").to(SurfaceAntecedent::Unit),
    ));
    let antecedent = antecedent_atom
        .clone()
        .then(
            just(Token::Ampersand)
                .ignore_then(antecedent_atom)
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

    // Both branches consume an event; residual syntax is one level in this fragment.
    // No recursive alternatives or optional separators are introduced.
    let conclusion = choice((
        antecedent
            .clone()
            .then_ignore(just(Token::Multimap))
            .then(event.clone())
            .map(|(required, event)| (Some(required), event)),
        event.clone().map(|event| (None, event)),
    ));

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
        .then(antecedent.clone())
        .then_ignore(just(Token::Leq))
        .then(conclusion)
        .map(|(((name, params), premise), (required, conclusion))| {
            FormalDeclaration::Rule(FormalRule {
                name,
                body: match required {
                    Some(required) => SurfaceRuleBody::Residual {
                        params,
                        premise,
                        required,
                        conclusion,
                    },
                    None => SurfaceRuleBody::Horn {
                        params,
                        premise,
                        conclusion,
                    },
                },
            })
        });

    let rule = choice((horn_rule, rule_application));

    let default_body = choice((
        keyword("supernormal")
            .ignore_then(ident.clone())
            .map(|rule| (SurfaceDefaultBody::Supernormal { rule }, None)),
        keyword("exception")
            .ignore_then(ident.clone())
            .then_ignore(keyword("to"))
            .then(ident.clone())
            .then(keyword("when").ignore_then(antecedent.clone()).or_not())
            .map(|((rule, to), condition)| (SurfaceDefaultBody::Exception { rule, to }, condition)),
    ));

    let default = keyword("default")
        .ignore_then(ident.clone())
        .then_ignore(just(Token::Equal))
        .then(default_body)
        .then(
            keyword("blocked")
                .then_ignore(keyword("when"))
                .ignore_then(antecedent)
                .or_not(),
        )
        .map(|((name, (body, condition)), blocking)| {
            FormalDeclaration::Default(FormalDefault {
                name,
                body,
                condition,
                blocking,
            })
        });

    // Each list item consumes an event; internal commas are mandatory. Only
    // the final comma is optional. Box this new declaration at its boundary
    // to keep its combinator type out of the existing declaration choice.
    let experiment = keyword("experiment")
        .ignore_then(ident.clone())
        .then(
            choice((
                keyword("seeds").to(ExperimentInputKind::Seeds),
                keyword("premises").to(ExperimentInputKind::Premises),
            ))
            .then(
                event
                    .clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then_ignore(keyword("query"))
            .then(choice((
                keyword("residual").to(ExperimentQueryKind::Residual),
                keyword("fourFusion").to(ExperimentQueryKind::FourFusion),
            )))
            .then(event.clone())
            .delimited_by(just(Token::LBrace), just(Token::RBrace)),
        )
        .map(|(name, (((input_kind, input), query_kind), goal))| {
            FormalDeclaration::Experiment(FormalExperiment {
                name,
                input_kind,
                input,
                query_kind,
                goal,
            })
        })
        .boxed();

    let declaration = choice((entity, const_decl, verb, rule, default, experiment));

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
            '#' => {
                for (_, c) in chars.by_ref() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                }
            }
            '/' => match chars.next() {
                Some((_, '/')) => {
                    for (_, c) in chars.by_ref() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                    }
                }
                Some((_, '*')) => {
                    let mut closed = false;
                    while let Some((_, c)) = chars.next() {
                        if c == '*' && matches!(chars.peek(), Some((_, '/'))) {
                            chars.next();
                            closed = true;
                            break;
                        }
                    }
                    if !closed {
                        return Err(Error::new(format!(
                            "unterminated block comment at byte {idx}"
                        )));
                    }
                }
                _ => return Err(Error::new(format!("unexpected `/` at byte {idx}"))),
            },
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
            '⊸' => tokens.push(Token::Multimap),
            '&' => tokens.push(Token::Ampersand),
            '~' => tokens.push(Token::Tilde),
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
                } else if matches!(chars.peek(), Some((_, 'o'))) {
                    chars.next();
                    tokens.push(Token::Multimap);
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
                "seeded" | "derived" | "effect" => {
                    declarations.push(FormalDeclaration::Verb(self.parse_verb()?))
                }
                "rule" => declarations.push(FormalDeclaration::Rule(self.parse_rule()?)),
                "default" => declarations.push(FormalDeclaration::Default(self.parse_default()?)),
                "experiment" => {
                    declarations.push(FormalDeclaration::Experiment(self.parse_experiment()?))
                }
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
            "derived" => SurfaceVerbKind::Derived,
            "effect" => SurfaceVerbKind::Effect,
            other => return Err(Error::new(format!("expected verb kind, got `{other}`"))),
        };
        self.expect_ident_value("verb")?;
        let name = self.expect_ident()?;
        let args = self.parse_args()?;
        let effect = if matches!(kind, SurfaceVerbKind::Effect | SurfaceVerbKind::Derived) {
            if self.eat(&Token::FatArrow) {
                Some(self.parse_effect()?)
            } else {
                None
            }
        } else {
            None
        };
        let program = if self.eat(&Token::Equal) {
            self.expect_ident_value("program")?;
            Some(self.expect_ident()?)
        } else {
            None
        };

        Ok(FormalVerb {
            name,
            kind,
            args,
            effect,
            program,
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
        } else if name == "PropositionTime" {
            Type::PropositionTime
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
            let right = self.parse_antecedent()?;
            let body = if self.eat(&Token::Multimap) {
                SurfaceRuleBody::Residual {
                    params,
                    premise,
                    required: right,
                    conclusion: self.parse_event()?,
                }
            } else {
                let SurfaceAntecedent::Event(conclusion) = right else {
                    return Err(Error::new("conjunctive consequent requires a residual"));
                };
                SurfaceRuleBody::Horn {
                    params,
                    premise,
                    conclusion,
                }
            };

            Ok(FormalRule { name, body })
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
        let mut antecedent = self.parse_antecedent_atom()?;
        while self.eat(&Token::Ampersand) {
            let right = self.parse_antecedent_atom()?;
            antecedent = SurfaceAntecedent::And(Box::new(antecedent), Box::new(right));
        }
        Ok(antecedent)
    }

    fn parse_antecedent_atom(&mut self) -> Result<SurfaceAntecedent, Error> {
        if matches!(self.tokens.get(self.pos), Some(Token::Ident(name)) if name == "I")
            && self.tokens.get(self.pos + 1) != Some(&Token::LParen)
        {
            self.pos += 1;
            Ok(SurfaceAntecedent::Unit)
        } else {
            self.parse_event().map(SurfaceAntecedent::Event)
        }
    }

    fn parse_default(&mut self) -> Result<FormalDefault, Error> {
        self.expect_ident_value("default")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Equal)?;
        let body = match self.expect_ident()?.as_str() {
            "supernormal" => SurfaceDefaultBody::Supernormal {
                rule: self.expect_ident()?,
            },
            "exception" => {
                let rule = self.expect_ident()?;
                self.expect_ident_value("to")?;
                SurfaceDefaultBody::Exception {
                    rule,
                    to: self.expect_ident()?,
                }
            }
            other => return Err(Error::new(format!("expected default kind, got `{other}`"))),
        };
        let condition =
            if matches!(body, SurfaceDefaultBody::Exception { .. }) && self.eat_ident("when") {
                Some(self.parse_antecedent()?)
            } else {
                None
            };
        let blocking = if self.eat_ident("blocked") {
            self.expect_ident_value("when")?;
            Some(self.parse_antecedent()?)
        } else {
            None
        };
        Ok(FormalDefault {
            name,
            body,
            condition,
            blocking,
        })
    }

    fn parse_event(&mut self) -> Result<SurfaceEvent, Error> {
        let polarity = if self.eat(&Token::Tilde) {
            Polarity::EvidentialNot
        } else {
            Polarity::Positive
        };
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

        Ok(SurfaceEvent {
            polarity,
            verb,
            args,
            time,
        })
    }

    fn parse_experiment(&mut self) -> Result<FormalExperiment, Error> {
        self.expect_ident_value("experiment")?;
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let input_kind = match self.expect_ident()?.as_str() {
            "seeds" => ExperimentInputKind::Seeds,
            "premises" => ExperimentInputKind::Premises,
            _ => return Err(Error::new("expected seeds or premises")),
        };
        self.expect(&Token::LBrace)?;
        let mut input = Vec::new();
        if !self.eat(&Token::RBrace) {
            loop {
                input.push(self.parse_event()?);
                if self.eat(&Token::RBrace) {
                    break;
                }
                self.expect(&Token::Comma)?;
                if self.eat(&Token::RBrace) {
                    break;
                }
            }
        }
        self.expect_ident_value("query")?;
        let query_kind = match self.expect_ident()?.as_str() {
            "residual" => ExperimentQueryKind::Residual,
            "fourFusion" => ExperimentQueryKind::FourFusion,
            _ => return Err(Error::new("expected residual or fourFusion")),
        };
        let goal = self.parse_event()?;
        self.expect(&Token::RBrace)?;
        Ok(FormalExperiment {
            name,
            input_kind,
            input,
            query_kind,
            goal,
        })
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
