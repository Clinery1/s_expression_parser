#[cfg(test)]
mod tests;


const NO_DATA:ObjectParseError=ObjectParseError::NoData;


enum State {
    Number,
    List,
    String,
    Ident,
}
#[derive(Debug,PartialEq,Copy,Clone)]
pub enum ObjectParseError {
    NoClosingParen,
    NoClosingQuote,
    NoData,
    ExtraClosingParen,
}
#[derive(Debug,PartialEq,Clone)]
pub enum Object<'input> {
    List(Vec<Self>),
    String(String),
    Ident(&'input str),
    Number(&'input str),
}
impl<'input> Object<'input> {
    pub fn str_data(&self)->Option<&str> {
        match self {
            Self::List(_)=>None,
            Self::String(s)=>Some(&s),
            Self::Ident(i)=>Some(i),
            Self::Number(n)=>Some(n),
        }
    }
    /// Parses the minimum amount of text to create an object.
    /// Returns the amount of characters used on success.
    pub fn from_str_inner(s:&'input str,index_start:usize)->Result<(Self,usize),Error> {
        let mut indices=s.char_indices().peekable();
        let mut state;
        let mut start;
        let mut count=0;
        let mut in_comment=false;
        let mut negative=false;
        loop {
            let (idx,c)=indices.next().ok_or(Error{index:index_start+count,err:NO_DATA})?;
            count+=1;
            start=idx;
            if in_comment {
                match c {
                    '\n'=>in_comment=false,
                    _=>{},
                }
            } else {
                match c {
                    '-'=>{
                        state=State::Number;
                        negative=true;
                        break;
                    },
                    '0'..='9'=>{
                        state=State::Number;
                        break;
                    },
                    ' '|'\t'|'\r'|'\n'=>{},
                    '('=>{
                        state=State::List;
                        break;
                    },
                    ')'=>return Err(Error{index:index_start+count,err:ObjectParseError::ExtraClosingParen}),
                    ';'=>in_comment=true,
                    '"'=>{
                        state=State::String;
                        break;
                    },
                    _=>{
                        state=State::Ident;
                        break;
                    },
                }
            }
        }
        loop {
            match state {
                State::Number=>{
                    let mut end=start;
                    if let Some((idx,c))=indices.peek() {
                        end=*idx;
                        match c {
                            '0'..='9'=>{},
                            '('|')'|' '|'\t'|'\r'|'\n'=>{
                                if negative {
                                    state=State::Ident;
                                    continue;
                                } else {
                                    return Ok((Object::Number(&s[start..end]),count));
                                }
                            },
                            _=>{
                                state=State::Ident;
                                continue;
                            },
                        }
                    }
                    for (idx,c) in indices {
                        end=idx;
                        match c {
                            '0'..='9'|'_'|'.'=>{},
                            _=>break,
                        }
                        count+=1;
                    }
                    return Ok((Object::Number(&s[start..end]),count));
                },
                State::List=>{
                    let mut items=Vec::new();
                    let mut good=false;
                    while let Some((idx,c))=indices.next() {
                        match c {
                            ')'=>{
                                good=true;
                                count+=1;
                                break;
                            },
                            ' '|'\t'|'\r'|'\n'=>count+=1,
                            _=>{
                                let (obj,len)=Object::from_str_inner(&s[idx..],index_start+count)?;
                                items.push(obj);
                                count+=len;
                                while let Some((idx,_))=indices.peek() {
                                    if *idx==count {
                                        break;
                                    } else {
                                        indices.next();
                                    }
                                }
                            },
                        }
                    }
                    if !good {
                        return Err(Error{index:index_start+count,err:ObjectParseError::NoClosingParen});
                    }
                    return Ok((Object::List(items),count));
                },
                State::String=>{
                    let mut s=String::new();
                    let mut escape=false;
                    let mut good=false;
                    for (_,c) in indices {
                        count+=1;
                        match c {
                            '\\'=>{
                                if escape {
                                    s.push('\\');
                                    escape=false;
                                } else {
                                    escape=true;
                                }
                            },
                            'n' if escape=>{s.push('\n');escape=false},
                            'r' if escape=>{s.push('\r');escape=false},
                            '"' if escape=>{s.push('"');escape=false},
                            't' if escape=>{s.push('t');escape=false},
                            '0' if escape=>{s.push('\0');escape=false},
                            '"' if !escape=>{
                                good=true;
                                break;
                            },
                            _=>{s.push(c);escape=false},
                        }
                    }
                    if !good {
                        return Err(Error{index:index_start+count,err:ObjectParseError::NoClosingParen});
                    }
                    return Ok((Object::String(s),count));
                },
                State::Ident=>{
                    let mut end=start;
                    for (idx,c) in indices {
                        end=idx;
                        match c {
                            ' '|'\r'|'\n'|'\t'|'('|')'|'"'=>break,
                            _=>{},
                        }
                        count+=1;
                    }
                    return Ok((Object::Ident(&s[start..end]),count));
                },
            }
        }
    }
    #[inline]
    pub fn from_str(s:&'input str)->Result<(Self,usize),Error> {Self::from_str_inner(s,0)}
}


#[derive(Debug)]
pub struct Error {
    pub index:usize,
    pub err:ObjectParseError,
}
impl Error {
    pub fn line_col(&self,source:&str)->Option<(usize,usize)> {
        if self.index>=source.len() {
            return None;
        }
        let mut line=0;
        let mut col=0;
        for (idx,c) in source.char_indices() {
            if idx>=self.index {
                break;
            }
            col+=1;
            match c {
                '\n'=>{
                    line+=1;
                    col=0;
                },
                _=>{},
            }
        }
        return Some((line,col));
    }
    pub fn error_line<'a>(&self,source:&'a str)->Option<&'a str> {
        if self.index>=source.len() {
            return None;
        }
        let mut last_line=0;
        for (idx,c) in source.char_indices() {
            if idx>=self.index {
                break;
            }
            match c {
                '\n'=>{
                    last_line=idx;
                },
                _=>{},
            }
        }
        let eol=source[self.index..].find('\n').unwrap_or(0)+self.index;
        return Some(&source[last_line..eol].trim());
    }
}
#[derive(Debug)]
pub struct File<'input> {
    pub items:Vec<Object<'input>>,
}
impl<'input> File<'input> {
    pub fn parse_file(contents:&'input str)->Result<Self,Error> {
        let mut items=Vec::new();
        let mut count=0;
        while count<contents.len() {
            match Object::from_str(&contents[count..]) {
                Ok((obj,len))=>{
                    items.push(obj);
                    count+=len;
                },
                Err(e)=>{
                    if e.err==ObjectParseError::NoData {
                        break;
                    } else {
                        return Err(e);
                    }
                },
            }
        }
        return Ok(File{items});
    }
}
