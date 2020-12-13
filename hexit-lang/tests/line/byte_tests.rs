// individual bytes
test_eval!(unit_00: "00" => Ok(vec![ 0x00 ]));
test_eval!(unit_FF: "FF" => Ok(vec![ 0xFF ]));
test_eval!(unit_ff: "ff" => Ok(vec![ 0xFF ]));
test_eval!(unit_fF: "fF" => Ok(vec![ 0xFF ]));
test_eval!(unit_Ff: "Ff" => Ok(vec![ 0xFF ]));
test_eval!(unit_f7: "f7" => Ok(vec![ 0xF7 ]));
test_eval!(unit_F7: "F7" => Ok(vec![ 0xF7 ]));

// pairs of bytes
test_eval!(pair:        "09F9"   => Ok(vec![ 0x09, 0xF9 ]));
test_eval!(pair_space:  "09 F9"  => Ok(vec![ 0x09, 0xF9 ]));
test_eval!(pair_spaces: "09  F9" => Ok(vec![ 0x09, 0xF9 ]));

// oh baby, a triple
test_eval!(triple:      "09F965" => Ok(vec![ 0x09, 0xF9, 0x65 ]));
