// repeating one byte
test_eval!(repeat_1:  "x1(AB)"  => Ok(vec![ 0xAB; 1 ]));
test_eval!(repeat_3:  "x3(AB)"  => Ok(vec![ 0xAB; 3 ]));
test_eval!(repeat_11: "x11(AB)" => Ok(vec![ 0xAB; 11 ]));

// repeating two bytes
test_eval!(repeat_22: "x11(AB AB)" => Ok(vec![ 0xAB; 22 ]));

// repeating three bytes
test_eval!(repeat_33: "x11(AB AB AB)" => Ok(vec![ 0xAB; 33 ]));
