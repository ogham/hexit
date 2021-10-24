test_eval!(json:      "\"JSON\""      => Ok(vec![ b'J', b'S', b'O', b'N' ]));
test_eval!(silence:   "\"\""          => Ok(vec![ ]));
test_eval!(backslash: "\"\\\\\""      => Ok(vec![ b'\\' ]));
test_eval!(backquote: "\"\\\"\""      => Ok(vec![ b'"' ]));
test_eval!(surround:  "AB\"JSON\"CD"  => Ok(vec![ 0xAB, b'J', b'S', b'O', b'N', 0xCD ]));
test_eval!(newline:   "\"hi\\nyo\""   => Ok(vec![ b'h', b'i', b'\n', b'y', b'o' ]));
test_eval!(rewline:   "\"hi\\ryo\""   => Ok(vec![ b'h', b'i', b'\r', b'y', b'o' ]));
test_eval!(tab:       "\"hi\\tyo\""   => Ok(vec![ b'h', b'i', b'\t', b'y', b'o' ]));
