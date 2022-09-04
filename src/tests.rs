use crate::*;


const fn loc(line:usize,column:usize,index:usize)->Location {Location{line,column,index}}
#[test]
fn test_string() {
    let contents=r##""Hello world!""##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==14);
    assert_eq!(obj,Object::String(loc(0,0,0),String::from(&contents[1..13]),loc(0,14,14)));
}
#[test]
fn test_number() {
    let contents=r##"1234 5678"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==4);
    let start=Location{line:0,column:0,index:0};
    let end=Location{line:0,column:4,index:4};
    assert_eq!(obj,Object::Number(start,&contents[start.index..end.index],end));
}
#[test]
fn test_ident() {
    let contents=r##"an_identifier another_one"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==13);
    let start=Location{line:0,column:0,index:0};
    let end=Location{line:0,column:13,index:13};
    assert_eq!(obj,Object::Ident(start,&contents[start.index..end.index],end));
}
#[test]
fn test_list1() {
    let contents=r##"(hello world)"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    dbg!(&obj);
    assert_eq!(obj,Object::List(Location{line:0,column:0,index:0},vec![
        Object::Ident(Location{line:0,column:1,index:1},"hello",Location{line:0,column:6,index:6}),
        Object::Ident(Location{line:0,column:7,index:7},"world",Location{line:0,column:12,index:12}),
    ],Location{line:0,column:13,index:13}));
}
#[test]
fn test_list2() {
    let contents=r##"(print "Hello, world!" 1234)"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    assert_eq!(obj,Object::List(loc(0,0,0),vec![
        Object::Ident(loc(0,1,1),"print",loc(0,6,6)),
        Object::String(loc(0,7,7),"Hello, world!".to_string(),loc(0,22,22)),
        Object::Number(loc(0,23,23),"1234",loc(0,27,27)),
    ],loc(0,28,28)));
}
#[test]
fn test_list3() {
    let contents=r##"(print
    "Hello, world!")"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    assert_eq!(obj,Object::List(loc(0,0,0),vec![
        Object::Ident(loc(0,1,1),"print",loc(0,6,6)),
        Object::String(loc(1,4,11),"Hello, world!".to_string(),loc(1,19,26)),
    ],loc(1,20,27)));
}
#[test]
fn test_list4() {
    let contents=
r##"(print
    (object
        (name "Clinery")
        (years_experience 6))
    1234)"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    let to_match=Object::List(loc(0,0,0),vec![
        Object::Ident(loc(0,1,1),"print",loc(0,6,6)),
        Object::List(loc(1,4,11),vec![
            Object::Ident(loc(1,5,12),"object",loc(1,11,18)),
            Object::List(loc(2,8,27),vec![
                Object::Ident(loc(2,9,28),"name",loc(2,13,32)),
                Object::String(loc(2,14,33),"Clinery".to_string(),loc(2,23,42)),
            ],loc(2,24,43)),
            Object::List(loc(3,8,52),vec![
                Object::Ident(loc(3,9,53),"years_experience",loc(3,25,69)),
                Object::Number(loc(3,26,70),"6",loc(3,27,71)),
            ],loc(3,28,72)),
        ],loc(3,29,73)),
        Object::Number(loc(4,4,78),"1234",loc(4,8,82)),
    ],loc(4,9,83));
    dbg!(&obj,&to_match);
    assert_eq!(obj,to_match);
}
