test_eval!(repeat_1:  "x1(AB)"   => Ok(vec![ 0xAB; 1 ]));
test_eval!(repeat_3:  "x3(AB)"   => Ok(vec![ 0xAB; 3 ]));
test_eval!(repeat_11: "x11(AB)"  => Ok(vec![ 0xAB; 11 ]));
