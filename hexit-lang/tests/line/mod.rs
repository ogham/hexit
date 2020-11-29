macro_rules! test_eval {
    ($name:ident: $input:expr => $result:expr) => {
        #[test]
        fn $name() {
            let program = hexit_lang::Program::read($input).expect("Parsing failed");
            let constants = hexit_lang::ConstantsTable::builtin_set();
            assert_eq!(program.run(constants, None), $result);
        }
    };
}


mod byte_tests;
mod constant_tests;
mod decimal_form_tests;
mod form_tests;
mod function_tests;
mod string_tests;
