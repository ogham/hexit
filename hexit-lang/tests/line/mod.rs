macro_rules! test_eval {
    ($name:ident: $input:expr => $result:expr) => {
        #[test]
        fn $name() {
            let program = hexit_lang::Program::read($input).expect("Parsing failed");
            let constants = hexit_lang::ConstantsTable::builtin_set();

            let result = program.run(&constants, None).map_err(|e| e.to_string());
            assert_eq!(result, $result);
        }
    };
}


mod bitwise_function_tests;
mod byte_tests;
mod constant_tests;
mod decimal_form_tests;
mod form_tests;
mod repeat_tests;
mod string_tests;
