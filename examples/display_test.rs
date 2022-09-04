use s_expression_parser::*;


fn main() {
    let contents=r#"((list "hello"))(object (name "Clinery") (languages_known (list "rust" "c" "assembly" "javascript" "etc")))"#;
    let parsed=File::parse_file(contents).unwrap();
    for item in parsed.items {
        println!("{}",item);
    }
}
