use nom::{
    branch::alt,
    number::complete::float,
    bytes::complete::tag,
    character::complete::{char},    
    multi::{separated_list, many1, many0},
    sequence::{separated_pair, preceded, pair, delimited},
    IResult,
};

// EVENTS
// An event is something like "sine;freq=100;dur=100" (an event type followed by a list of parameters)
// or just the event type.

// param names can be fixed for now ... 
pub fn param_name(input: &str) -> IResult<&str, &str> {
    alt((tag("pitch"), tag("dur"), tag("lvl"), tag("rate"), tag("lp-freq"), tag("lp-dist")))(input)
}

pub fn param(input: &str) -> IResult<&str, (&str, f32)> {
    separated_pair(param_name, char('='), float)(input)
}

pub fn param_list(input: &str) -> IResult<&str, Vec<(&str, f32)>> {
    separated_list(tag(";"), param)(input)
}

// for custom sample events, this would need to be replaced by a freeform string function ... 
pub fn event_name(input: &str) -> IResult<&str, &str> {
    alt((tag("sine"), tag("sqare"), tag("sqr"), tag("hh"), tag("bd"), tag("sn"), tag("~")))(input)
}

// sine;freq=100.0;dur=200
pub fn event_with_param(input: &str) -> IResult<&str, (&str, Vec<(&str, f32)>)> {
    pair(event_name, preceded(char(';'), param_list))(input)
}

// sine
pub fn event_without_param(input: &str) -> IResult<&str, (&str, Vec<(&str, f32)>)> {
    let res = event_name(input)?;
    Ok((res.0, (res.1, Vec::new())))
}

// both of the former
pub fn event(input: &str) -> IResult<&str, (&str, Vec<(&str, f32)>)> {
    alt((event_with_param, event_without_param))(input)
}

pub fn event_pattern(input: &str) -> IResult<&str, Vec<(&str, Vec<(&str, f32)>)>> {
    separated_list(many1(char(' ')), event)(input)
}

// SEQ GENS
pub fn pattern_func_name(input: &str) -> IResult<&str, &str> {
    alt((tag("rnd"), tag("cyc"), tag("learn")))(input)
}

pub fn param_func_name(input: &str) -> IResult<&str, &str> {
    alt((tag("bounce"), tag("brownian")))(input)
}

pub fn func_name(input: &str) -> IResult<&str, &str> {
    alt((param_func_name, pattern_func_name))(input)
}

pub fn pattern_func(input: &str) -> IResult<&str, (&str, Vec<(&str, Vec<(&str, f32)>)>)> {
    separated_pair(func_name, delimited(many0(char(' ')), tag(">>"), many0(char(' '))), event_pattern)(input)
}

pub fn param_func_header(input: &str) -> IResult<&str, &str> {
    preceded(tag("@"), param_name)(input)
}

pub fn param_func(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(param_func_header, delimited(many0(char(' ')), char(':'), many0(char(' '))), func_name)(input)
}

pub fn param_func_with_values(input: &str) -> IResult<&str, ((&str, &str), Vec<f32>)> {
    separated_pair(param_func, delimited(many0(char(' ')), tag(">>"), many0(char(' '))), separated_list(many1(char(' ')), float))(input)
}

pub fn pattern_line(input: &str) -> IResult<&str, ((&str, Vec<(&str, Vec<(&str, f32)>)>), Vec<((&str, &str), Vec<f32>)>)> {
    separated_pair(pattern_func, many0(char(' ')), separated_list(many1(char(' ')), param_func_with_values))(input)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
        
    #[test]
    fn test_pattern_func() {
        let res = pattern_func("rnd >> bd ~ ~ sn ~ ~");       
        println!("Result: {:?}", res);
        assert!(!res.is_err());
    }

    #[test]
    fn test_pattern_line_without_params() {
        let res = pattern_line("rnd >> bd ~ ~ sn ~ ~");
        println!("Result: {:?}", res);        
        assert!(!res.is_err());
    }

    #[test]
    fn test_pattern_line_with_one_param() {
        let res = pattern_line("rnd >> bd ~ ~ ~ sn ~ ~ ~ @rate: cyc >> 1.0 0.9 0.6 0.4");
        println!("Result: {:?}", res);        
        assert!(!res.is_err());
    }

    #[test]
    fn test_param_func() {
        let res = param_func_with_values("@rate: rnd >> 1.0 0.9 0.6 0.4");
        println!("Result: {:?}", res);
        assert!(!res.is_err());
    }

    #[test]
    fn test_param_func_header() {
        let res = param_func_header("@rate");
        println!("Result: {:?}", res);
        assert!(!res.is_err());
    }
}


