use std::fmt::{
    Display,
    Debug,
    Formatter,
    Result as FmtResult,
};


#[cfg(test)]
mod tests;


macro_rules! terminals {
    ()=>{
        '('|')'|' '|'\t'|'\r'|'\n'|'"'
    };
}


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
#[derive(PartialEq,Clone)]
pub enum Object<'input> {
    List(Location,Vec<Self>,Location),
    String(Location,String,Location),
    Ident(Location,&'input str,Location),
    Number(Location,&'input str,Location),
}
impl<'input> Display for Object<'input> {
    fn fmt(&self,f:&mut Formatter)->FmtResult {
        use Object::*;
        if let Some(width)=f.width() {
            for _ in 0..width {
                f.write_str(" ")?;
            }
        }
        match self {
            List(_,items,_)=>{
                match items.len() {
                    0=>write!(f,"()")?,
                    1=>write!(f,"({})",items[0])?,
                    2 if !items[1].is_list()=>write!(f,"({} {})",items[0],items[1])?,
                    _=>{
                        writeln!(f,"({}",items[0])?;
                        let last=items.len()-1;
                        for (i,item) in items.iter().enumerate().skip(1) {
                            let mut width=4;
                            if let Some(f_width)=f.width() {
                                width+=f_width;
                            }
                            write!(f,"{:1$}",item,width)?;
                            if i!=last {
                                writeln!(f)?;
                            }
                        }
                        write!(f,")")?;
                    },
                }
            },
            Ident(_,s,_)|Number(_,s,_)=>f.write_str(s)?,
            String(_,s,_)=>write!(f,"{:?}",s)?,
        }
        return Ok(());
    }
}
impl<'input> Debug for Object<'input> {
    fn fmt(&self,f:&mut Formatter)->FmtResult {
        use Object::*;
        if f.alternate() {
            if let Some(width)=f.width() {
                for _ in 0..width {
                    f.write_str(" ")?;
                }
            }
        }
        match self {
            List(_,items,_)=>{
                if f.alternate() {
                    writeln!(f,"List([")?;
                } else {
                    write!(f,"List([")?;
                }
                let last=items.len()-1;
                for (i,item) in items.iter().enumerate() {
                    if f.alternate() {
                        let mut width=4;
                        if let Some(f_width)=f.width() {
                            width+=f_width;
                            for _ in 0..f_width {
                                f.write_str(" ")?;
                            }
                        }
                        write!(f,"{:#1$?}",item,width)?;
                        if i!=last {
                            writeln!(f,",")?;
                        } else {
                            writeln!(f)?;
                        }
                    } else {
                        write!(f,"{:?}",item)?;
                        if i!=last {
                            write!(f,",")?;
                        }
                    }
                }
                write!(f,"])")?;
            },
            Ident(_,s,_)=>write!(f,"Ident({})",s)?,
            Number(_,s,_)=>write!(f,"Number({})",s)?,
            String(_,s,_)=>write!(f,"String({:?})",s)?,
        }
        return Ok(());
    }
}
impl<'input> Object<'input> {
    pub fn is_list(&self)->bool {
        match self {
            Self::List(..)=>true,
            _=>false,
        }
    }
    pub fn str_data(&self)->Option<&str> {
        match self {
            Self::List(..)=>None,
            Self::String(_,s,_)=>Some(&s),
            Self::Ident(_,i,_)=>Some(i),
            Self::Number(_,n,_)=>Some(n),
        }
    }
    /// Parses the minimum amount of text to create an object.
    /// Returns the amount of characters used on success.
    fn from_str_inner(s:&'input str,location:&mut Location)->Result<(Self,usize),Error> {
        let mut indices=s.char_indices().peekable();
        let mut state;
        let mut start;
        let mut count=0;
        let mut in_comment=false;
        let mut negative=false;
        let start_loc=*location;
        loop {
            let (idx,c)=indices.next().ok_or(Error{loc:*location,err:NO_DATA})?;
            count+=1;
            start=idx;
            location.index+=1;
            location.column+=1;
            if in_comment {
                match c {
                    '\n'=>{
                        in_comment=false;
                        location.line+=1;
                        location.column=0;
                    },
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
                    '\n'=>{
                        location.line+=1;
                        location.column=0;
                    },
                    ' '|'\t'|'\r'=>{},
                    '('=>{
                        state=State::List;
                        break;
                    },
                    ')'=>return Err(Error{loc:*location,err:ObjectParseError::ExtraClosingParen}),
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
        'main:loop {
            match state {
                State::Number=>{
                    let mut end=start;
                    if let Some((idx,c))=indices.peek() {
                        end=*idx;
                        match c {
                            '0'..='9'=>{},
                            terminals!()=>{
                                if negative {
                                    state=State::Ident;
                                    continue;
                                } else {
                                    return Ok((Object::Number(start_loc,&s[start..end],*location),count));
                                }
                            },
                            _=>{
                                state=State::Ident;
                                continue;
                            },
                        }
                    }
                    while let Some((idx,c))=indices.next() {
                        end=idx;
                        match c {
                            terminals!()=>break,
                            '0'..='9'|'_'|'.'=>{
                                location.index+=1;
                                location.column+=1;
                                count+=1;
                            },
                            _=>{
                                location.index+=1;
                                location.column+=1;
                                count+=1;
                                state=State::Ident;
                                continue 'main;
                            },
                        }
                    }
                    return Ok((Object::Number(start_loc,&s[start..end],*location),count));
                },
                State::List=>{
                    let mut items=Vec::new();
                    let mut good=false;
                    while let Some((idx,c))=indices.next() {
                        match c {
                            ')'=>{
                                good=true;
                                count+=1;
                                location.index+=1;
                                location.column+=1;
                                break;
                            },
                            '\n'=>{
                                count+=1;
                                location.line+=1;
                                location.index+=1;
                                location.column=0;
                            },
                            ' '|'\t'|'\r'=>{
                                count+=1;
                                location.index+=1;
                                location.column+=1;
                            },
                            _=>{
                                // eprintln!("List item before: {:?}",*location);
                                // let item_start_idx=location.index;
                                let (obj,len)=Object::from_str_inner(&s[idx..],location)?;
                                // eprintln!("List item after: {:?}; Contents: `{}`",*location,&s[item_start_idx..location.index]);
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
                        return Err(Error{loc:*location,err:ObjectParseError::NoClosingParen});
                    }
                    return Ok((Object::List(start_loc,items,*location),count));
                },
                State::String=>{
                    let mut s=String::new();
                    let mut escape=false;
                    let mut good=false;
                    for (_,c) in indices {
                        count+=1;
                        let mut inc_col=true;
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
                            't' if escape=>{s.push('t');escape=false},
                            '0' if escape=>{s.push('\0');escape=false},
                            '"'=>if escape {
                                s.push('"');
                                escape=false;
                            } else {
                                good=true;
                                location.index+=1;
                                location.column+=1;
                                break;
                            },
                            '\n'=>{
                                location.line+=1;
                                location.column=0;
                                inc_col=false;
                            },
                            _=>{s.push(c);escape=false},
                        }
                        location.index+=1;
                        if inc_col {location.column+=1}
                    }
                    if !good {
                        return Err(Error{loc:*location,err:ObjectParseError::NoClosingParen});
                    }
                    return Ok((Object::String(start_loc,s,*location),count));
                },
                State::Ident=>{
                    let mut end=start;
                    for (idx,c) in indices {
                        end=idx;
                        match c {
                            terminals!()=>break,
                            _=>{},
                        }
                        count+=1;
                        location.index+=1;
                        location.column+=1;
                    }
                    return Ok((Object::Ident(start_loc,&s[start..end],*location),count));
                },
            }
        }
    }
    #[inline]
    pub fn from_str(s:&'input str)->Result<(Self,usize),Error> {Self::from_str_inner(s,&mut Location::default())}
}


#[derive(Debug)]
pub struct Error {
    pub loc:Location,
    pub err:ObjectParseError,
}
impl Error {
    pub fn error_line<'a>(&self,source:&'a str)->Option<&'a str> {
        if self.loc.index>=source.len() {
            return None;
        }
        let mut last_line=0;
        for (idx,c) in source.char_indices() {
            if idx>=self.loc.index {
                break;
            }
            match c {
                '\n'=>{
                    last_line=idx;
                },
                _=>{},
            }
        }
        let eol=source[self.loc.index..].find('\n').unwrap_or(0)+self.loc.index;
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
        let mut location=Location::default();
        while count<contents.len() {
            match Object::from_str_inner(&contents[count..],&mut location) {
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
#[derive(Copy,Clone,Debug,PartialEq,Default)]
pub struct Location {
    pub line:usize,
    pub column:usize,
    pub index:usize,
}
impl Location {
}
