// ‘and’ function
test_eval!(and_1byte:  "and(5C)"     => Ok(vec![ 0x5C ]));
test_eval!(and_2bytes: "and(5C 74)"  => Ok(vec![ 0x54 ]));

// ‘or’ function
test_eval!(or_1byte:   "or(5C)"      => Ok(vec![ 0x5C ]));
test_eval!(or_2bytes:  "or(5C 74)"   => Ok(vec![ 0x7C ]));

// ‘xor’ function
test_eval!(xor_1byte:  "xor(5C)"     => Ok(vec![ 0x5C ]));
test_eval!(xor_2bytes: "xor(5C 74)"  => Ok(vec![ 0x28 ]));

// ‘not’ function
test_eval!(not_1bype:  "not(5C)"     => Ok(vec![ 0xA3 ]));
