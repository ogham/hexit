macro_rules! test_eval {
    ($name:ident: $input:expr => $result:expr) => {
        #[test]
        fn $name() {
            let program = hexit_lang::Program::read($input).expect("Parsing failed");
            let constants = hexit_lang::constants::Table::builtin_set();

            let result = program.run(&constants, None).map_err(|e| e.to_string());
            assert_eq!(result, $result);
        }
    };
}


mod bit_form_tests;
mod bitwise_function_tests;
mod byte_tests;
mod constant_tests;
mod decimal_form_tests;
mod float_form_tests;
mod form_tests;
mod repeat_tests;
mod string_tests;
