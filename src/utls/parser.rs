use std::fmt;
use std::error::Error;

pub struct Parser;

#[derive(Debug)]
pub struct ParserError {
    details: String
}

pub struct ParserResult {
    pub url : String,
    pub stdin : String,
    pub target : String,
    pub code : String,
    pub options : Vec<String>,
}
impl ParserError {
    fn new(msg: &str) -> ParserError {
        ParserError{details: msg.to_string()}
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl Parser {
    pub async fn get_components(input : &str) -> Result<ParserResult, ParserError> {

        let mut result = ParserResult {
            url : Default::default(),
            stdin : Default::default(),
            target : Default::default(),
            code : Default::default(),
            options : Default::default()
        };


        let code_block : usize;
        if let Some(index) = input.find("`") {
            code_block = index;
        }
        else {
            code_block = input.len();
        }

        let mut args : Vec<&str> = input[..code_block].split(" ").collect();
        args.remove(0); // ditch command str (;compile, ;asm)
        result.target = args.remove(0).trim().to_owned();

        let mut iter = args.iter();
        while let Some(c) = iter.next() {
            if c.contains("```"){
                break;
            }

            if *c == "<" {
                let link = match iter.next() {
                    Some(link) => link,
                    None => return Err(ParserError::new("'<' operator requires a url\n\nUsage: `;compile c++ < http://foo.bar/code.txt`"))
                };
                result.url = link.trim().to_string();
            }
            else if *c == "|" {
                let mut input : String = String::new();
                while let Some(stdin) = iter.next() {
                    if stdin.contains("```") {
                        break;
                    }
                    if *stdin == "<" {
                        return Err(ParserError::new("`|`` operator should be last, unable to continue"))
                    }
                    input.push_str(stdin);
                    input.push_str(" ");
                }

                result.stdin = input.trim().to_owned();
            }
            else {
                result.options.push(c.trim().to_string());
            }
        }

        if !result.url.is_empty() {
            let response = match reqwest::get(&result.url).await {
                Ok(b) => b,
                Err(_e) => return Err(ParserError::new("GET request failed, perhaps your link is unreachable?"))
            };

            let body = match response.text().await {
                Ok(t) => t,
                Err(_e) => return Err(ParserError::new("Unable to grab resource"))
            };

            result.code = body;
        }
        else {
            Parser::find_code_block(&mut result, input)?;
        }

        if !result.url.is_empty() && !result.code.is_empty() {
            result.stdin = result.code;
            result.code = String::new();
        }
        Ok(result)
    }

    fn find_code_block(result : & mut ParserResult, haystack : &str) -> Result<(), ParserError> {
        use regex::Regex;
        let re = Regex::new(r"```[\S\s]*?\n([\s\S]*?)```").unwrap();
        let matches = re.captures_iter(haystack);

        let mut captures : Vec<&str> = Vec::new();
        let list =  matches.enumerate();
        for (_, cap) in list {
            captures.push(cap.get(1).unwrap().as_str());
        }

        if captures.len() > 1 {
            result.code = String::from(captures[1]);
            result.stdin = String::from(captures[0]);
        }
        else if captures.len() == 1 {
            result.code = String::from(captures[0]);
        } else {
            return Err(ParserError::new("You must attach a code-block containing code to your message"))
        }
        Ok(())
    }
}