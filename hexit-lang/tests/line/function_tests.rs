test_eval!(be32_fifty: "be32[50]" => Ok(vec![ 0, 0, 0, 50 ]));

test_eval!(repeat_1:  "x1(AB)"     => Ok(vec![ 0xAB; 1 ]));
test_eval!(repeat_3:  "x3(AB)"     => Ok(vec![ 0xAB; 3 ]));
test_eval!(repeat_11: "x11(AB)"    => Ok(vec![ 0xAB; 11 ]));
