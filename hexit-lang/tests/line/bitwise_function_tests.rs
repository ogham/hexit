// ‘and’ function
test_eval!(and_1byte:  "and(5C)"     => Ok(vec![ 0x5C ]));
test_eval!(and_2bytes: "and(5C 74)"  => Ok(vec![ 0x54 ]));
