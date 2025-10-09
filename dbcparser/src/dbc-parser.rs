/*
 * Copyright (C) 2018 Marcel Buesing (MIT License)
 * Origin: https://github.com/marcelbuesing/can-dbc
 *
 * Adaptation (2022) to Redpesk and LibAfb model
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

use crate::data::*;
use std::str;

use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_till, take_while, take_while1},
    character::complete::{self, char, line_ending, multispace0, space0, space1},
    combinator::{map, opt, value},
    error::{ErrorKind, ParseError},
    multi::{many0, many_till, separated_list0},
    number::complete::double,
    sequence::preceded,
    AsChar, IResult, InputTakeAtPosition,
};

fn is_semi_colon(chr: char) -> bool {
    chr == ';'
}

fn is_c_string_char(chr: char) -> bool {
    chr.is_digit(10) || chr.is_alphabetic() || chr == '_'
}

fn is_c_ident_head(chr: char) -> bool {
    chr.is_alphabetic() || chr == '_'
}

fn is_quote(chr: char) -> bool {
    chr == '"'
}

/// Multispace zero or more
fn ms0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
{
    input.split_at_position_complete(|item| {
        let c = item.as_char();
        c != ' '
    })
}

/// Multi space one or more
fn ms1<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
{
    input.split_at_position1_complete(
        |item| {
            let c = item.as_char();
            c != ' '
        },
        ErrorKind::MultiSpace,
    )
}

/// Colon aka `:`
fn colon(s: &str) -> IResult<&str, char> {
    char(':')(s)
}

/// Comma aka ','
fn comma(s: &str) -> IResult<&str, char> {
    char(',')(s)
}

/// Comma aka ';'
fn semi_colon(s: &str) -> IResult<&str, char> {
    char(';')(s)
}

/// Quote aka '"'
fn quote(s: &str) -> IResult<&str, char> {
    char('"')(s)
}

/// Pipe character
fn pipe(s: &str) -> IResult<&str, char> {
    char('|')(s)
}

/// at character
fn at(s: &str) -> IResult<&str, char> {
    char('@')(s)
}

/// brace open aka '('
fn brc_open(s: &str) -> IResult<&str, char> {
    char('(')(s)
}

/// brace close aka ')'
fn brc_close(s: &str) -> IResult<&str, char> {
    char(')')(s)
}

/// bracket open aka '['
fn brk_open(s: &str) -> IResult<&str, char> {
    char('[')(s)
}

/// bracket close aka ']'
fn brk_close(s: &str) -> IResult<&str, char> {
    char(']')(s)
}

/// A valid C_identifier. C_identifiers start with a  alphacharacter or an underscore
/// and may further consist of alpha­numeric, characters and underscore
fn c_ident(s: &str) -> IResult<&str, String> {
    let (s, head) = take_while1(is_c_ident_head)(s)?;
    let (s, remaining) = take_while(is_c_string_char)(s)?;
    Ok((s, [head, remaining].concat()))
}

fn c_ident_vec(s: &str) -> IResult<&str, Vec<String>> {
    separated_list0(comma, c_ident)(s)
}

fn char_string(s: &str) -> IResult<&str, &str> {
    let (s, _) = quote(s)?;
    let (s, char_string_value) = take_till(|c| is_quote(c as char))(s)?;
    let (s, _) = quote(s)?;
    Ok((s, char_string_value))
}

fn little_endian(s: &str) -> IResult<&str, ByteOrder> {
    map(char('1'), |_| ByteOrder::LittleEndian)(s)
}

fn big_endian(s: &str) -> IResult<&str, ByteOrder> {
    map(char('0'), |_| ByteOrder::BigEndian)(s)
}

fn byte_order(s: &str) -> IResult<&str, ByteOrder> {
    alt((little_endian, big_endian))(s)
}

fn message_id(s: &str) -> IResult<&str, MessageId> {
    map(complete::u32, MessageId)(s)
}

fn signed(s: &str) -> IResult<&str, ValueType> {
    map(char('-'), |_| ValueType::Signed)(s)
}

fn unsigned(s: &str) -> IResult<&str, ValueType> {
    map(char('+'), |_| ValueType::Unsigned)(s)
}

fn value_type(s: &str) -> IResult<&str, ValueType> {
    alt((signed, unsigned))(s)
}

fn multiplexer(s: &str) -> IResult<&str, MultiplexIndicator> {
    let (s, _) = ms1(s)?;
    let (s, _) = char('m')(s)?;
    let (s, d) = complete::u64(s)?;
    let (s, _) = ms1(s)?;
    Ok((s, MultiplexIndicator::MultiplexedSignal(d)))
}

fn multiplexor(s: &str) -> IResult<&str, MultiplexIndicator> {
    let (s, _) = ms1(s)?;
    let (s, _) = char('M')(s)?;
    let (s, _) = ms1(s)?;
    Ok((s, MultiplexIndicator::Multiplexor))
}

fn multiplexor_and_multiplexed(s: &str) -> IResult<&str, MultiplexIndicator> {
    let (s, _) = ms1(s)?;
    let (s, _) = char('m')(s)?;
    let (s, d) = complete::u64(s)?;
    let (s, _) = char('M')(s)?;
    let (s, _) = ms1(s)?;
    Ok((s, MultiplexIndicator::MultiplexorAndMultiplexedSignal(d)))
}

fn plain(s: &str) -> IResult<&str, MultiplexIndicator> {
    let (s, _) = ms1(s)?;
    Ok((s, MultiplexIndicator::Plain))
}

fn multiplexer_indicator(s: &str) -> IResult<&str, MultiplexIndicator> {
    alt((multiplexer, multiplexor, multiplexor_and_multiplexed, plain))(s)
}

fn version(s: &str) -> IResult<&str, Version> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("VERSION")(s)?;
    let (s, _) = ms1(s)?;
    let (s, v) = char_string(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, Version(v.to_string())))
}

fn bit_timing(s: &str) -> IResult<&str, Vec<Baudrate>> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BS_:")(s)?;
    let (s, baudrates) =
        opt(preceded(ms1, separated_list0(comma, map(complete::u64, Baudrate))))(s)?;
    Ok((s, baudrates.unwrap_or_default()))
}

fn signal(s: &str) -> IResult<&str, Signal> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("SG_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, name) = c_ident(s)?;
    let (s, multiplexer_indicator) = multiplexer_indicator(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, start_bit) = complete::u64(s)?;
    let (s, _) = pipe(s)?;
    let (s, size) = complete::u64(s)?;
    let (s, _) = at(s)?;
    let (s, byte_order) = byte_order(s)?;
    let (s, value_type) = value_type(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = brc_open(s)?;
    let (s, factor) = double(s)?;
    let (s, _) = comma(s)?;
    let (s, offset) = double(s)?;
    let (s, _) = brc_close(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = brk_open(s)?;
    let (s, min) = double(s)?;
    let (s, _) = pipe(s)?;
    let (s, max) = double(s)?;
    let (s, _) = brk_close(s)?;
    let (s, _) = ms1(s)?;
    let (s, unit) = char_string(s)?;
    let (s, _) = ms1(s)?;
    let (s, receivers) = c_ident_vec(s)?;
    let (s, _) = line_ending(s)?;
    Ok((
        s,
        Signal {
            name,
            multiplexer_indicator,
            start_bit,
            size,
            byte_order,
            value_type,
            factor,
            offset,
            min,
            max,
            unit: unit.to_string(),
            receivers,
        },
    ))
}

fn message(s: &str) -> IResult<&str, Message> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BO_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_name) = c_ident(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_size) = complete::u64(s)?;
    let (s, _) = ms1(s)?;
    let (s, transmitter) = transmitter(s)?;
    let (s, signals) = many0(signal)(s)?;
    Ok((
        s,
        (Message {
            id: message_id,
            name: message_name,
            size: message_size,
            transmitter: transmitter,
            signals: signals,
        }),
    ))
}

fn attribute_default(s: &str) -> IResult<&str, AttributeDefault> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BA_DEF_DEF_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, attribute_name) = char_string(s)?;
    let (s, _) = ms1(s)?;
    let (s, attribute_value) = attribute_value(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;

    Ok((s, AttributeDefault { attribute_name: attribute_name.to_string(), attribute_value }))
}

fn node_comment(s: &str) -> IResult<&str, Comment> {
    let (s, _) = tag("BU_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, node_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, comment) = char_string(s)?;

    Ok((s, Comment::Node { node_name, comment: comment.to_string() }))
}

fn message_comment(s: &str) -> IResult<&str, Comment> {
    let (s, _) = tag("BO_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, comment) = char_string(s)?;

    Ok((s, Comment::Message { message_id, comment: comment.to_string() }))
}

fn signal_comment(s: &str) -> IResult<&str, Comment> {
    let (s, _) = tag("SG_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, comment) = char_string(s)?;
    Ok((s, Comment::Signal { message_id, signal_name, comment: comment.to_string() }))
}

fn env_var_comment(s: &str) -> IResult<&str, Comment> {
    let (s, _) = ms0(s)?;
    let (s, _) = tag("EV_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, comment) = char_string(s)?;
    Ok((s, Comment::EnvVar { env_var_name, comment: comment.to_string() }))
}

fn comment_plain(s: &str) -> IResult<&str, Comment> {
    let (s, comment) = char_string(s)?;
    Ok((s, Comment::Plain { comment: comment.to_string() }))
}

fn comment(s: &str) -> IResult<&str, Comment> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("CM_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, comment) =
        alt((node_comment, message_comment, env_var_comment, signal_comment, comment_plain))(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, comment))
}

fn value_description(s: &str) -> IResult<&str, ValDescription> {
    let (s, a) = double(s)?;
    let (s, _) = ms1(s)?;
    let (s, b) = char_string(s)?;
    Ok((s, ValDescription { a, b: b.to_string() }))
}

fn value_description_for_signal(s: &str) -> IResult<&str, ValueDescription> {
    let (s, _) = ms0(s)?;
    let (s, _) = tag("VAL_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_name) = c_ident(s)?;
    let (s, value_descriptions) =
        many_till(preceded(ms1, value_description), preceded(opt(ms1), semi_colon))(s)?;
    Ok((
        s,
        ValueDescription::Signal {
            message_id,
            signal_name,
            value_descriptions: value_descriptions.0,
        },
    ))
}

fn value_description_for_env_var(s: &str) -> IResult<&str, ValueDescription> {
    let (s, _) = ms0(s)?;
    let (s, _) = tag("VAL_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var_name) = c_ident(s)?;
    let (s, value_descriptions) =
        many_till(preceded(ms1, value_description), preceded(opt(ms1), semi_colon))(s)?;
    Ok((
        s,
        ValueDescription::EnvironmentVariable {
            env_var_name,
            value_descriptions: value_descriptions.0,
        },
    ))
}

fn value_descriptions(s: &str) -> IResult<&str, ValueDescription> {
    let (s, _) = multispace0(s)?;
    let (s, vd) = alt((value_description_for_signal, value_description_for_env_var))(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, vd))
}

fn env_float(s: &str) -> IResult<&str, EnvType> {
    value(EnvType::EnvTypeFloat, char('0'))(s)
}

fn env_int(s: &str) -> IResult<&str, EnvType> {
    value(EnvType::EnvTypeu64, char('1'))(s)
}

fn env_data(s: &str) -> IResult<&str, EnvType> {
    value(EnvType::EnvTypeu64, char('2'))(s)
}

fn env_var_type(s: &str) -> IResult<&str, EnvType> {
    alt((env_float, env_int, env_data))(s)
}

fn dummy_node_vector_0(s: &str) -> IResult<&str, AccessType> {
    value(AccessType::DummyNodeVector0, char('0'))(s)
}

fn dummy_node_vector_1(s: &str) -> IResult<&str, AccessType> {
    value(AccessType::DummyNodeVector1, char('1'))(s)
}

fn dummy_node_vector_2(s: &str) -> IResult<&str, AccessType> {
    value(AccessType::DummyNodeVector2, char('2'))(s)
}
fn dummy_node_vector_3(s: &str) -> IResult<&str, AccessType> {
    value(AccessType::DummyNodeVector3, char('3'))(s)
}

fn access_type(s: &str) -> IResult<&str, AccessType> {
    let (s, _) = tag("DUMMY_NODE_VECTOR")(s)?;
    let (s, node) =
        alt((dummy_node_vector_0, dummy_node_vector_1, dummy_node_vector_2, dummy_node_vector_3))(
            s,
        )?;
    Ok((s, node))
}

fn access_node_vector_xxx(s: &str) -> IResult<&str, AccessNode> {
    value(AccessNode::AccessNodeVectorXXX, tag("VECTOR_XXX"))(s)
}

fn access_node_name(s: &str) -> IResult<&str, AccessNode> {
    map(c_ident, AccessNode::AccessNodeName)(s)
}

fn access_node(s: &str) -> IResult<&str, AccessNode> {
    alt((access_node_vector_xxx, access_node_name))(s)
}

/// Environment Variable Definitions
fn environment_variable(s: &str) -> IResult<&str, EnvironmentVariable> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("EV_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var_name) = c_ident(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var_type) = env_var_type(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = brk_open(s)?;
    let (s, min) = complete::i64(s)?;
    let (s, _) = pipe(s)?;
    let (s, max) = complete::i64(s)?;
    let (s, _) = brk_close(s)?;
    let (s, _) = ms1(s)?;
    let (s, unit) = char_string(s)?;
    let (s, _) = ms1(s)?;
    let (s, initial_value) = double(s)?;
    let (s, _) = ms1(s)?;
    let (s, ev_id) = complete::i64(s)?;
    let (s, _) = ms1(s)?;
    let (s, access_type) = access_type(s)?;
    let (s, _) = ms1(s)?;
    let (s, access_nodes) = separated_list0(comma, access_node)(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((
        s,
        EnvironmentVariable {
            env_var_name,
            env_var_type,
            min,
            max,
            unit: unit.to_string(),
            initial_value,
            ev_id,
            access_type,
            access_nodes,
        },
    ))
}

fn environment_variable_data(s: &str) -> IResult<&str, EnvironmentVariableData> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("ENVVAR_DATA_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var_name) = c_ident(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, data_size) = complete::u64(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, EnvironmentVariableData { env_var_name, data_size }))
}

fn signal_type(s: &str) -> IResult<&str, SignalType> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("SGTYPE_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_type_name) = c_ident(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, size) = complete::u64(s)?;
    let (s, _) = at(s)?;
    let (s, byte_order) = byte_order(s)?;
    let (s, value_type) = value_type(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = brc_open(s)?;
    let (s, factor) = double(s)?;
    let (s, _) = comma(s)?;
    let (s, offset) = double(s)?;
    let (s, _) = brc_close(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = brk_open(s)?;
    let (s, min) = double(s)?;
    let (s, _) = pipe(s)?;
    let (s, max) = double(s)?;
    let (s, _) = brk_close(s)?;
    let (s, _) = ms1(s)?;
    let (s, unit) = char_string(s)?;
    let (s, _) = ms1(s)?;
    let (s, default_value) = double(s)?;
    let (s, _) = ms1(s)?;
    let (s, value_table) = c_ident(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((
        s,
        SignalType {
            signal_type_name,
            size,
            byte_order,
            value_type,
            factor,
            offset,
            min,
            max,
            unit: unit.to_string(),
            default_value,
            value_table,
        },
    ))
}

#[allow(dead_code)]
fn attribute_value_uint64(s: &str) -> IResult<&str, AttributeValue> {
    map(complete::u64, AttributeValue::AttributeValueU64)(s)
}

#[allow(dead_code)]
fn attribute_value_int64(s: &str) -> IResult<&str, AttributeValue> {
    map(complete::i64, AttributeValue::AttributeValueI64)(s)
}

fn attribute_value_f64(s: &str) -> IResult<&str, AttributeValue> {
    map(double, AttributeValue::AttributeValueF64)(s)
}

fn attribute_value_charstr(s: &str) -> IResult<&str, AttributeValue> {
    map(char_string, |x| AttributeValue::AttributeValueCharString(x.to_string()))(s)
}

fn attribute_value(s: &str) -> IResult<&str, AttributeValue> {
    alt((
        // attribute_value_uint64,
        // attribute_value_int64,
        attribute_value_f64,
        attribute_value_charstr,
    ))(s)
}

fn network_node_attribute_value(s: &str) -> IResult<&str, AttributeValuedForObjectType> {
    let (s, _) = tag("BU_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, node_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, value) = attribute_value(s)?;
    Ok((s, AttributeValuedForObjectType::NetworkNodeAttributeValue(node_name, value)))
}

fn message_definition_attribute_value(s: &str) -> IResult<&str, AttributeValuedForObjectType> {
    let (s, _) = tag("BO_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, value) = opt(attribute_value)(s)?;
    Ok((s, AttributeValuedForObjectType::MessageDefinitionAttributeValue(message_id, value)))
}

fn signal_attribute_value(s: &str) -> IResult<&str, AttributeValuedForObjectType> {
    let (s, _) = tag("SG_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, value) = attribute_value(s)?;
    Ok((s, AttributeValuedForObjectType::SignalAttributeValue(message_id, signal_name, value)))
}

fn env_variable_attribute_value(s: &str) -> IResult<&str, AttributeValuedForObjectType> {
    let (s, _) = tag("EV_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, value) = attribute_value(s)?;
    Ok((s, AttributeValuedForObjectType::EnvVariableAttributeValue(env_var_name, value)))
}

fn raw_attribute_value(s: &str) -> IResult<&str, AttributeValuedForObjectType> {
    map(attribute_value, AttributeValuedForObjectType::RawAttributeValue)(s)
}

fn attribute_value_for_object(s: &str) -> IResult<&str, AttributeValueForObject> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BA_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, attribute_name) = char_string(s)?;
    let (s, _) = ms1(s)?;
    let (s, attribute_value) = alt((
        network_node_attribute_value,
        message_definition_attribute_value,
        signal_attribute_value,
        env_variable_attribute_value,
        raw_attribute_value,
    ))(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, AttributeValueForObject { attribute_name: attribute_name.to_string(), attribute_value }))
}

// TODO add properties
fn attribute_definition_node(s: &str) -> IResult<&str, AttributeDefinition> {
    let (s, _) = tag("BU_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, node) = take_till(|c| is_semi_colon(c as char))(s)?;
    Ok((s, AttributeDefinition::Node(node.to_string())))
}

// TODO add properties
fn attribute_definition_signal(s: &str) -> IResult<&str, AttributeDefinition> {
    let (s, _) = tag("SG_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal) = take_till(|c| is_semi_colon(c as char))(s)?;
    Ok((s, AttributeDefinition::Signal(signal.to_string())))
}

// TODO add properties
fn attribute_definition_environment_variable(s: &str) -> IResult<&str, AttributeDefinition> {
    let (s, _) = tag("EV_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, env_var) = take_till(|c| is_semi_colon(c as char))(s)?;
    Ok((s, AttributeDefinition::EnvironmentVariable(env_var.to_string())))
}

// TODO add properties
fn attribute_definition_message(s: &str) -> IResult<&str, AttributeDefinition> {
    let (s, _) = tag("BO_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message) = take_till(|c| is_semi_colon(c as char))(s)?;
    Ok((s, AttributeDefinition::Message(message.to_string())))
}

// TODO add properties
fn attribute_definition_plain(s: &str) -> IResult<&str, AttributeDefinition> {
    let (s, plain) = take_till(|c| is_semi_colon(c as char))(s)?;
    Ok((s, AttributeDefinition::Plain(plain.to_string())))
}

fn attribute_definition(s: &str) -> IResult<&str, AttributeDefinition> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BA_DEF_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, def) = alt((
        attribute_definition_node,
        attribute_definition_signal,
        attribute_definition_environment_variable,
        attribute_definition_message,
        attribute_definition_plain,
    ))(s)?;

    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, def))
}

fn symbol(s: &str) -> IResult<&str, Symbol> {
    let (s, _) = space1(s)?;
    let (s, symbol) = c_ident(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, Symbol(symbol)))
}

fn new_symbols(s: &str) -> IResult<&str, Vec<Symbol>> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("NS_ :")(s)?;
    let (s, _) = space0(s)?;
    let (s, _) = line_ending(s)?;
    let (s, symbols) = many0(symbol)(s)?;
    Ok((s, symbols))
}

/// Network node
fn node(s: &str) -> IResult<&str, Node> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BU_:")(s)?;
    let (s, li) = opt(preceded(ms1, separated_list0(ms1, c_ident)))(s)?;
    let (s, _) = space0(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, Node(li.unwrap_or_default())))
}

fn signal_type_ref(s: &str) -> IResult<&str, SignalTypeRef> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("SGTYPE_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_type_name) = c_ident(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, SignalTypeRef { message_id, signal_name, signal_type_name }))
}

fn value_table(s: &str) -> IResult<&str, ValueTable> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("VAL_TABLE_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, value_table_name) = c_ident(s)?;
    let (s, value_descriptions) =
        many_till(preceded(ms0, value_description), preceded(ms0, semi_colon))(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, ValueTable { value_table_name, value_descriptions: value_descriptions.0 }))
}

fn extended_multiplex_mapping(s: &str) -> IResult<&str, ExtendedMultiplexMapping> {
    let (s, _) = ms0(s)?;
    let (s, min_value) = complete::u64(s)?;
    let (s, _) = char('-')(s)?;
    let (s, max_value) = complete::u64(s)?;
    Ok((s, ExtendedMultiplexMapping { min_value, max_value }))
}

fn extended_multiplex(s: &str) -> IResult<&str, ExtendedMultiplex> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("SG_MUL_VAL_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, multiplexor_signal_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, mappings) = separated_list0(tag(","), extended_multiplex_mapping)(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, ExtendedMultiplex { message_id, signal_name, multiplexor_signal_name, mappings }))
}

fn signed_or_unsigned_integer(s: &str) -> IResult<&str, SignalExtendedValueType> {
    value(SignalExtendedValueType::SignedOrUnsignedInteger, tag("0"))(s)
}
fn ieee_float_32bit(s: &str) -> IResult<&str, SignalExtendedValueType> {
    value(SignalExtendedValueType::IEEEfloat32Bit, tag("1"))(s)
}
fn ieee_double_64bit(s: &str) -> IResult<&str, SignalExtendedValueType> {
    value(SignalExtendedValueType::IEEEdouble64bit, tag("2"))(s)
}

fn signal_extended_value_type(s: &str) -> IResult<&str, SignalExtendedValueType> {
    alt((signed_or_unsigned_integer, ieee_float_32bit, ieee_double_64bit))(s)
}

fn signal_extended_value_type_list(s: &str) -> IResult<&str, SignalExtendedValueTypeList> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("SIG_VALTYPE_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = opt(colon)(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_extended_value_type) = signal_extended_value_type(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, SignalExtendedValueTypeList { message_id, signal_name, signal_extended_value_type }))
}

fn transmitter_vector_xxx(s: &str) -> IResult<&str, Transmitter> {
    value(Transmitter::VectorXXX, tag("Vector__XXX"))(s)
}

fn transmitter_node_name(s: &str) -> IResult<&str, Transmitter> {
    map(c_ident, Transmitter::NodeName)(s)
}

fn transmitter(s: &str) -> IResult<&str, Transmitter> {
    alt((transmitter_vector_xxx, transmitter_node_name))(s)
}

fn message_transmitters(s: &str) -> IResult<&str, Vec<Transmitter>> {
    separated_list0(comma, transmitter)(s)
}

fn message_transmitter(s: &str) -> IResult<&str, MessageTransmitter> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("BO_TX_BU_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, transmitter) = message_transmitters(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, MessageTransmitter { message_id, transmitter }))
}

fn signal_groups(s: &str) -> IResult<&str, SignalGroups> {
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("SIG_GROUP_")(s)?;
    let (s, _) = ms1(s)?;
    let (s, message_id) = message_id(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_group_name) = c_ident(s)?;
    let (s, _) = ms1(s)?;
    let (s, repetitions) = complete::u64(s)?;
    let (s, _) = ms1(s)?;
    let (s, _) = colon(s)?;
    let (s, _) = ms1(s)?;
    let (s, signal_names) = separated_list0(ms1, c_ident)(s)?;
    let (s, _) = semi_colon(s)?;
    let (s, _) = line_ending(s)?;
    Ok((s, SignalGroups { message_id, signal_group_name, repetitions, signal_names }))
}

pub fn dbc_from_str(dbc_str: &str) -> Result<DbcObject, DbcError<'_>> {
    match dbc_parse_str(dbc_str) {
        Ok((remaining, object)) => {
            match multispace0::<&str, ()>(remaining) {
                Ok((ascii, _)) => {
                    if !ascii.is_empty() {
                        println!("Unprocessed DBC: {}", ascii);
                        return Err(DbcError {
                            uid: "parsing-not-completed",
                            error: Error::Incomplete(remaining),
                            info: "fail to parse dbc input".to_owned(),
                        });
                    }
                    return Ok(object);
                }
                Err(_error) => {
                    return Err(DbcError {
                        uid: "parsing-not-clean",
                        error: Error::Misc,
                        info: "fail to parse dbc input".to_owned(),
                    });
                }
            };
        }
        Err(error) => {
            return Err(DbcError {
                uid: "parsing-fail",
                error: Error::Parsing(error.to_string()),
                info: "fail to parse full dbc input".to_owned(),
            });
        }
    }
}

fn dbc_parse_str(dbc_str: &str) -> IResult<&str, DbcObject> {
    let (
        dbc_str,
        (
            version,
            new_symbols,
            bit_timing,
            nodes,
            value_tables,
            messages,
            message_transmitters,
            environment_variables,
            environment_variable_data,
            signal_types,
            comments,
            attribute_definitions,
            attribute_defaults,
            attribute_values,
            value_descriptions,
            signal_type_refs,
            signal_groups,
            signal_extended_value_type_list,
            extended_multiplex,
        ),
    ) = permutation((
        version,
        new_symbols,
        opt(bit_timing),
        many0(node),
        many0(value_table),
        many0(message),
        many0(message_transmitter),
        many0(environment_variable),
        many0(environment_variable_data),
        many0(signal_type),
        many0(comment),
        many0(attribute_definition),
        many0(attribute_default),
        many0(attribute_value_for_object),
        many0(value_descriptions),
        many0(signal_type_ref),
        many0(signal_groups),
        many0(signal_extended_value_type_list),
        many0(extended_multiplex),
    ))(dbc_str)?;

    let (dbc_str, _) = multispace0(dbc_str)?;

    // return parser DBC object
    Ok((
        dbc_str,
        DbcObject {
            version,
            new_symbols,
            bit_timing,
            nodes,
            value_tables,
            messages,
            message_transmitters,
            environment_variables,
            environment_variable_data,
            signal_types,
            comments,
            attribute_definitions,
            attribute_defaults,
            attribute_values,
            value_descriptions,
            signal_type_refs,
            signal_groups,
            signal_extended_value_type_list,
            extended_multiplex,
        },
    ))
}
