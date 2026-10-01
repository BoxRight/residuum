use crate::elab::Error;
use crate::surface::{
    CnlConst, CnlDeclaration, CnlDefault, CnlEntity, CnlField, CnlModule, CnlRule, CnlVerb,
    FormalArg, SurfaceAntecedent, SurfaceDefaultBody, SurfaceEffect, SurfaceEvent,
    SurfaceFieldAccess, SurfaceStateOperation, SurfaceTerm, SurfaceTime, SurfaceVerbKind,
};
use crate::typed::Type;

pub fn parse_cnl(source: &str) -> Result<CnlModule, Error> {
    let mut lines = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());

    let module_line = lines
        .next()
        .ok_or_else(|| Error::new("expected `module` line"))?;
    let name = module_line
        .strip_prefix("module ")
        .ok_or_else(|| Error::new("expected `module NAME`"))?
        .to_string();

    let mut declarations = Vec::new();
    let collected: Vec<_> = lines.collect();
    let mut index = 0;
    while index < collected.len() {
        let line = collected[index];
        if line.starts_with("there is an entity called ") {
            declarations.push(CnlDeclaration::Entity(parse_bare_entity(line)?));
            index += 1;
        } else if line.starts_with("there is a ") && line.contains(" called ") {
            declarations.push(CnlDeclaration::Const(parse_const(line)?));
            index += 1;
        } else if line.starts_with("a ") && line.contains(" is a ") {
            declarations.push(CnlDeclaration::Entity(parse_entity(line)?));
            index += 1;
        } else if line.starts_with("a ")
            && (line.contains(" has a set of ") || line.contains(" has a "))
        {
            declarations.push(CnlDeclaration::Field(parse_field(line)?));
            index += 1;
        } else if line.starts_with('"') && line.ends_with("means:") {
            let (verb, consumed) = parse_verb_block(&collected[index..])?;
            declarations.push(CnlDeclaration::Verb(verb));
            index += consumed;
        } else if line.starts_with("rule ") {
            let (rule, consumed) = parse_rule_block(&collected[index..])?;
            declarations.push(CnlDeclaration::Rule(rule));
            index += consumed;
        } else if line.starts_with("default ") {
            declarations.push(CnlDeclaration::Default(parse_default(line)?));
            index += 1;
        } else {
            return Err(Error::new(format!("unexpected CNL line `{line}`")));
        }
    }

    Ok(CnlModule { name, declarations })
}

fn parse_const(line: &str) -> Result<CnlConst, Error> {
    let line = trim_period(line);
    let rest = line
        .strip_prefix("there is a ")
        .ok_or_else(|| Error::new("expected constant declaration"))?;
    let (ty, name) = split_once(rest, " called ")?;
    Ok(CnlConst {
        name: name.to_string(),
        ty: Type::Entity(ty.to_string()),
    })
}

fn parse_entity(line: &str) -> Result<CnlEntity, Error> {
    let line = trim_period(line);
    let rest = line
        .strip_prefix("a ")
        .ok_or_else(|| Error::new("expected entity declaration to start with `a`"))?;
    let (name, supertype) = split_once(rest, " is a ")?;

    Ok(CnlEntity {
        name: name.to_string(),
        supertypes: vec![supertype.to_string()],
    })
}

fn parse_bare_entity(line: &str) -> Result<CnlEntity, Error> {
    let name = trim_period(line)
        .strip_prefix("there is an entity called ")
        .ok_or_else(|| Error::new("expected bare entity declaration"))?;

    Ok(CnlEntity {
        name: name.to_string(),
        supertypes: Vec::new(),
    })
}

fn parse_field(line: &str) -> Result<CnlField, Error> {
    let line = trim_period(line);
    let rest = line
        .strip_prefix("a ")
        .ok_or_else(|| Error::new("expected field declaration to start with `a`"))?;
    let (owner, rest) = split_once(rest, " has a ")?;
    let (ty, field_name) = split_once(rest, " called ")?;
    let ty = if let Some(optional_item) = ty.strip_prefix("optional ") {
        Type::Optional(Box::new(Type::Entity(optional_item.to_string())))
    } else if let Some(plural_item) = ty.strip_prefix("set of ") {
        Type::Set(Box::new(Type::Entity(singularize(plural_item))))
    } else {
        Type::Entity(ty.to_string())
    };

    Ok(CnlField {
        owner: owner.to_string(),
        name: field_name.to_string(),
        ty,
    })
}

fn parse_verb_block(lines: &[&str]) -> Result<(CnlVerb, usize), Error> {
    let header = lines
        .first()
        .ok_or_else(|| Error::new("expected verb header"))?;
    let name = header
        .strip_prefix('"')
        .and_then(|rest| rest.split_once('"').map(|(name, _)| name))
        .ok_or_else(|| Error::new("expected quoted verb name"))?
        .to_string();
    let signature = *lines
        .get(1)
        .ok_or_else(|| Error::new("expected verb signature line"))?;

    let args = parse_signature(&name, signature)?;
    let effect_object_ty = args
        .iter()
        .find(|arg| arg.name == "object")
        .or_else(|| args.iter().find(|arg| arg.name == "conduct"))
        .and_then(|arg| match &arg.ty {
            Type::Entity(name) => Some(name.as_str()),
            _ => None,
        })
        .ok_or_else(|| Error::new("expected object-like verb argument"))?;

    let mut consumed = 2;

    let effect_lines = lines
        .iter()
        .copied()
        .skip(2)
        .take_while(|line| line.starts_with("causing ") || line.starts_with("and causing "))
        .collect::<Vec<_>>();
    let effect = if effect_lines.is_empty() {
        None
    } else {
        consumed = 2 + effect_lines.len();
        Some(parse_effect(&effect_lines.join(" "), effect_object_ty)?)
    };

    let kind = if effect.is_some() {
        SurfaceVerbKind::Effect
    } else {
        SurfaceVerbKind::Seeded
    };

    Ok((
        CnlVerb {
            name,
            kind,
            args,
            effect,
        },
        consumed,
    ))
}

fn parse_signature(verb_name: &str, line: &str) -> Result<Vec<FormalArg>, Error> {
    let line = trim_sentence_end(line);
    let rest = line
        .strip_prefix("a ")
        .ok_or_else(|| Error::new("expected verb signature to start with `a`"))?;
    let (subject_ty, rest) = split_verb_signature(rest, verb_name)?;
    let rest = strip_article(rest)?;

    if matches!(verb_name, "do" | "notDo") {
        let (conduct_ty, rest) = split_once(rest, " with ")?;
        let rest = strip_article(rest)?;
        let (indirect_ty, beneficiary_ty) = rest
            .split_once(" for a ")
            .ok_or_else(|| Error::new("expected conduct verb beneficiary"))?;
        let indirect_ty = indirect_ty
            .strip_prefix("optional ")
            .map(|ty| Type::Optional(Box::new(Type::Entity(ty.to_string()))))
            .unwrap_or_else(|| Type::Entity(indirect_ty.to_string()));
        return Ok(vec![
            FormalArg {
                name: "subject".to_string(),
                ty: Type::Entity(subject_ty.to_string()),
            },
            FormalArg {
                name: "conduct".to_string(),
                ty: Type::Entity(conduct_ty.to_string()),
            },
            FormalArg {
                name: "indirectObject".to_string(),
                ty: indirect_ty,
            },
            FormalArg {
                name: "beneficiary".to_string(),
                ty: Type::Entity(beneficiary_ty.to_string()),
            },
        ]);
    }

    let mut args = vec![
        FormalArg {
            name: "subject".to_string(),
            ty: Type::Entity(subject_ty.to_string()),
        },
        FormalArg {
            name: "object".to_string(),
            ty: Type::Entity(
                rest.split_once(" to a ")
                    .map(|(object_ty, _)| object_ty)
                    .unwrap_or(rest)
                    .to_string(),
            ),
        },
    ];

    if let Some((_, recipient_ty)) = rest.split_once(" to a ") {
        args.push(FormalArg {
            name: "recipient".to_string(),
            ty: Type::Entity(recipient_ty.to_string()),
        });
    }

    Ok(args)
}

fn split_verb_signature<'a>(rest: &'a str, verb_name: &str) -> Result<(&'a str, &'a str), Error> {
    if verb_name == "do" {
        return split_once(rest, " does ").or_else(|_| split_once(rest, " do "));
    }

    split_once(rest, &format!(" {verb_name} "))
        .or_else(|_| split_once(rest, &format!(" {verb_name}s ")))
}

fn parse_effect(line: &str, object_ty: &str) -> Result<SurfaceEffect, Error> {
    let line = trim_sentence_end(line);
    if line
        == "causing the object to be removed from the subject's assets, and added to the recipient's assets"
    {
        return Ok(SurfaceEffect {
            operations: vec![
                SurfaceStateOperation::RemoveFromSet {
                    target: SurfaceFieldAccess {
                        base: "subject".to_string(),
                        path: vec!["assets".to_string()],
                    },
                    value: SurfaceTerm::Var("object".to_string()),
                },
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "recipient".to_string(),
                        path: vec!["assets".to_string()],
                    },
                    value: SurfaceTerm::Var("object".to_string()),
                },
            ],
        });
    }
    if line
        == "causing the relation to be added to the subject's patrimony obligations, and added to the recipient's patrimony rights"
    {
        return Ok(SurfaceEffect {
            operations: vec![
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "subject".to_string(),
                        path: vec!["patrimony".to_string(), "obligations".to_string()],
                    },
                    value: SurfaceTerm::Var("object".to_string()),
                },
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "recipient".to_string(),
                        path: vec!["patrimony".to_string(), "rights".to_string()],
                    },
                    value: SurfaceTerm::Var("object".to_string()),
                },
            ],
        });
    }
    let relation_literal =
        |thing: SurfaceTerm, conduct: SurfaceTerm, creditor: &str| SurfaceTerm::RecordLiteral {
            entity: "Relation".to_string(),
            fields: vec![
                crate::surface::SurfaceRecordFieldValue {
                    field: "debtor".to_string(),
                    value: SurfaceTerm::Var("subject".to_string()),
                },
                crate::surface::SurfaceRecordFieldValue {
                    field: "thing".to_string(),
                    value: thing,
                },
                crate::surface::SurfaceRecordFieldValue {
                    field: "conduct".to_string(),
                    value: conduct,
                },
                crate::surface::SurfaceRecordFieldValue {
                    field: "creditor".to_string(),
                    value: SurfaceTerm::Var(creditor.to_string()),
                },
            ],
        };
    if line
        == "causing a Relation with debtor subject, thing object, conduct none, and creditor recipient to be added to the subject's patrimony obligations"
    {
        return Ok(SurfaceEffect {
            operations: vec![SurfaceStateOperation::AddToSet {
                target: SurfaceFieldAccess {
                    base: "subject".to_string(),
                    path: vec!["patrimony".to_string(), "obligations".to_string()],
                },
                value: relation_literal(
                    SurfaceTerm::Var("object".to_string()),
                    SurfaceTerm::None,
                    "recipient",
                ),
            }],
        });
    }
    if line
        == "causing a Relation with debtor subject, thing object, conduct none, and creditor recipient to be added to the subject's patrimony obligations, and causing a Relation with debtor subject, thing object, conduct none, and creditor recipient to be added to the recipient's patrimony rights"
    {
        return Ok(SurfaceEffect {
            operations: vec![
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "subject".to_string(),
                        path: vec!["patrimony".to_string(), "obligations".to_string()],
                    },
                    value: relation_literal(
                        SurfaceTerm::Var("object".to_string()),
                        SurfaceTerm::None,
                        "recipient",
                    ),
                },
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "recipient".to_string(),
                        path: vec!["patrimony".to_string(), "rights".to_string()],
                    },
                    value: relation_literal(
                        SurfaceTerm::Var("object".to_string()),
                        SurfaceTerm::None,
                        "recipient",
                    ),
                },
            ],
        });
    }
    if line
        == "causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the subject's patrimony obligations, and causing a Relation with debtor subject, thing indirectObject, conduct conduct, and creditor beneficiary to be added to the beneficiary's patrimony rights"
    {
        return Ok(SurfaceEffect {
            operations: vec![
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "subject".to_string(),
                        path: vec!["patrimony".to_string(), "obligations".to_string()],
                    },
                    value: relation_literal(
                        SurfaceTerm::Var("indirectObject".to_string()),
                        SurfaceTerm::Var("conduct".to_string()),
                        "beneficiary",
                    ),
                },
                SurfaceStateOperation::AddToSet {
                    target: SurfaceFieldAccess {
                        base: "beneficiary".to_string(),
                        path: vec!["patrimony".to_string(), "rights".to_string()],
                    },
                    value: relation_literal(
                        SurfaceTerm::Var("indirectObject".to_string()),
                        SurfaceTerm::Var("conduct".to_string()),
                        "beneficiary",
                    ),
                },
            ],
        });
    }

    let expected = format!("causing that {object_ty} to be added to that ");
    let rest = line
        .strip_prefix(&expected)
        .ok_or_else(|| Error::new("expected add-to-set effect sentence"))?;
    let (owner_ty, field) = split_once(rest, "'s ")?;

    if owner_ty != "Person" {
        return Err(Error::new(format!("unsupported effect owner `{owner_ty}`")));
    }

    Ok(SurfaceEffect {
        operations: vec![SurfaceStateOperation::AddToSet {
            target: SurfaceFieldAccess {
                base: "subject".to_string(),
                path: vec![field.to_string()],
            },
            value: SurfaceTerm::Var("object".to_string()),
        }],
    })
}

fn parse_rule_block(lines: &[&str]) -> Result<(CnlRule, usize), Error> {
    let header = lines
        .first()
        .ok_or_else(|| Error::new("expected rule header"))?;
    let name = header
        .strip_prefix("rule ")
        .and_then(|rest| rest.strip_suffix(':'))
        .ok_or_else(|| Error::new("expected `rule NAME:`"))?
        .to_string();
    if let Some(application) = lines
        .get(1)
        .copied()
        .filter(|line| line.starts_with("use "))
    {
        let application = trim_sentence_end(application)
            .strip_prefix("use ")
            .ok_or_else(|| Error::new("expected rule application"))?;
        let (rule, args) = split_once(application, " with ")?;
        return Ok((
            CnlRule {
                name,
                body: crate::surface::SurfaceRuleBody::Application {
                    rule: rule.to_string(),
                    args: args.split(", ").map(str::to_string).collect(),
                },
            },
            2,
        ));
    }

    let mut premise_lines = Vec::new();
    let mut index = 1;
    while let Some(line) = lines.get(index).copied() {
        if line.starts_with("when ") || line.starts_with("and when ") {
            premise_lines.push(line);
            index += 1;
        } else {
            break;
        }
    }
    if premise_lines.is_empty() {
        return Err(Error::new("expected rule premise line"));
    }
    let conclusion_line = *lines
        .get(index)
        .ok_or_else(|| Error::new("expected rule conclusion line"))?;

    Ok((
        CnlRule {
            name,
            body: crate::surface::SurfaceRuleBody::Horn {
                params: Vec::new(),
                premise: parse_rule_antecedent(&premise_lines)?,
                conclusion: parse_rule_conclusion(conclusion_line)?,
            },
        },
        index + 1,
    ))
}

fn parse_rule_antecedent(lines: &[&str]) -> Result<SurfaceAntecedent, Error> {
    let mut events = lines
        .iter()
        .map(|line| parse_rule_premise(line).map(SurfaceAntecedent::Event));
    let first = events
        .next()
        .ok_or_else(|| Error::new("expected rule premise line"))??;
    events.try_fold(first, |left, right| {
        Ok(SurfaceAntecedent::And(Box::new(left), Box::new(right?)))
    })
}

fn parse_rule_premise(line: &str) -> Result<SurfaceEvent, Error> {
    let line = trim_sentence_end(line)
        .strip_prefix("and when a ")
        .or_else(|| trim_sentence_end(line).strip_prefix("when a "))
        .ok_or_else(|| Error::new("expected rule premise to start with `when a`"))?
        .to_string();
    let (actor, rest, verb) = if let Some((actor, rest)) = line.split_once(" delivers ") {
        (actor, rest, "delivers")
    } else if let Some((actor, rest)) = line.split_once(" transfers ") {
        (actor, rest, "transfers")
    } else if let Some((actor, rest)) = line.split_once(" agreesPrice ") {
        (actor, rest, "agreesPrice")
    } else if let Some((actor, rest)) = line.split_once(" agreesObject ") {
        (actor, rest, "agreesObject")
    } else {
        return Err(Error::new(format!("unsupported rule premise `{line}`")));
    };
    let rest = strip_article(rest)?;
    let (object, rest) = split_once(rest, " to ")?;
    let rest = strip_article(rest)?;
    let (receiver, time) = split_once(rest, " at time ")?;

    Ok(SurfaceEvent {
        verb: verb.to_string(),
        polarity: crate::typed::Polarity::Positive,
        args: vec![actor.to_string(), object.to_string(), receiver.to_string()],
        time: Some(SurfaceTime::At(time.to_string())),
    })
}

fn parse_rule_conclusion(line: &str) -> Result<SurfaceEvent, Error> {
    let line = trim_sentence_end(line);
    let rest = line
        .strip_prefix("the ")
        .ok_or_else(|| Error::new("expected rule conclusion to start with `the`"))?;
    if let Some((receiver, rest)) = rest.split_once(" acquires the ") {
        let (object, time) = split_once(rest, " after ")?;
        return Ok(SurfaceEvent {
            verb: "acquires".to_string(),
            polarity: crate::typed::Polarity::Positive,
            args: vec![receiver.to_string(), object.to_string()],
            time: Some(SurfaceTime::After(time.to_string())),
        });
    }

    if let Some((giver, rest)) = rest.split_once(" transfers the ") {
        let (object, rest) = split_once(rest, " to the ")?;
        let (receiver, time) = split_once(rest, " after ")?;
        return Ok(SurfaceEvent {
            verb: "transfers".to_string(),
            polarity: crate::typed::Polarity::Positive,
            args: vec![giver.to_string(), object.to_string(), receiver.to_string()],
            time: Some(SurfaceTime::After(time.to_string())),
        });
    }

    let (giver, rest) = split_once(rest, " gives the ")?;
    let (object, rest) = split_once(rest, " to the ")?;
    let (receiver, time) = split_once(rest, " after ")?;

    Ok(SurfaceEvent {
        verb: "give".to_string(),
        polarity: crate::typed::Polarity::Positive,
        args: vec![giver.to_string(), object.to_string(), receiver.to_string()],
        time: Some(SurfaceTime::After(time.to_string())),
    })
}

fn parse_default(line: &str) -> Result<CnlDefault, Error> {
    let line = trim_period(line);
    let rest = line
        .strip_prefix("default ")
        .ok_or_else(|| Error::new("expected default declaration"))?;
    let (name, rest) = split_once(rest, ": normally use ")?;
    Ok(CnlDefault {
        name: name.to_string(),
        body: SurfaceDefaultBody::Supernormal {
            rule: rest.to_string(),
        },
    })
}

fn trim_sentence_end(line: &str) -> &str {
    line.trim_end_matches(['.', ','])
}

fn trim_period(line: &str) -> &str {
    line.trim_end_matches('.')
}

fn split_once<'a>(value: &'a str, delimiter: &str) -> Result<(&'a str, &'a str), Error> {
    value
        .split_once(delimiter)
        .ok_or_else(|| Error::new(format!("expected `{delimiter}` in `{value}`")))
}

fn strip_article(value: &str) -> Result<&str, Error> {
    value
        .strip_prefix("a ")
        .or_else(|| value.strip_prefix("an "))
        .or_else(|| value.strip_prefix("the "))
        .ok_or_else(|| Error::new(format!("expected article in `{value}`")))
}

fn singularize(value: &str) -> String {
    value.strip_suffix('s').unwrap_or(value).to_string()
}
