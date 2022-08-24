use crate::*;


#[test]
fn test_string() {
    let contents=r##""Hello world!""##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==14);
    assert_eq!(obj,Object::String(String::from("Hello world!")));
}
#[test]
fn test_number() {
    let contents=r##"1234 5678"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==4);
    assert_eq!(obj,Object::Number("1234"));
}
#[test]
fn test_ident() {
    let contents=r##"an_identifier another_one"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==13);
    assert_eq!(obj,Object::Ident("an_identifier"));
}
#[test]
fn test_list1() {
    let contents=r##"(hello world)"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    assert_eq!(obj,Object::List(vec![
        Object::Ident("hello"),
        Object::Ident("world"),
    ]));
}
#[test]
fn test_list2() {
    let contents=r##"(print "Hello, world!" 1234)"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    assert_eq!(obj,Object::List(vec![
        Object::Ident("print"),
        Object::String("Hello, world!".to_string()),
        Object::Number("1234"),
    ]));
}
#[test]
fn test_list3() {
    let contents=r##"(print (object (name "Clinery") (years_experience 6)) 1234)"##;
    let (obj,len)=Object::from_str(contents).unwrap();
    assert!(len==contents.len());
    assert_eq!(obj,Object::List(vec![
        Object::Ident("print"),
        Object::List(vec![
            Object::Ident("object"),
            Object::List(vec![
                Object::Ident("name"),
                Object::String("Clinery".to_string()),
            ]),
            Object::List(vec![
                Object::Ident("years_experience"),
                Object::Number("6"),
            ]),
        ]),
        Object::Number("1234"),
    ]));
}
