test_eval!(json:      "\"JSON\""  => Ok(vec![ b'J', b'S', b'O', b'N' ]));
test_eval!(silence:   "\"\""      => Ok(vec![ ]));
test_eval!(backslash: "\"\\\\\""  => Ok(vec![ b'\\' ]));
test_eval!(backquote: "\"\\\"\""  => Ok(vec![ b'"' ]));
test_eval!(surround:  "AB\"JSON\"CD"  => Ok(vec![ 0xAB, b'J', b'S', b'O', b'N', 0xCD ]));
