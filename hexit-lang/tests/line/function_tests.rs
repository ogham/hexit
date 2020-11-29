// be32 function
test_eval!(be32_0:   "be32[0]"           => Ok(vec![ 0, 0, 0, 0 ]));
test_eval!(be32_50:  "be32[50]"          => Ok(vec![ 0, 0, 0, 50 ]));
test_eval!(be32_255: "be32[255]"         => Ok(vec![ 0, 0, 0, 255 ]));
test_eval!(be32_256: "be32[256]"         => Ok(vec![ 0, 0, 1, 0 ]));
test_eval!(be32_all: "be32[4294967295]"  => Ok(vec![ 255, 255, 255, 255 ]));

// le32 function
test_eval!(le32_0:   "le32[0]"           => Ok(vec![ 0, 0, 0, 0 ]));
test_eval!(le32_50:  "le32[50]"          => Ok(vec![ 50, 0, 0, 0 ]));
test_eval!(le32_255: "le32[255]"         => Ok(vec![ 255, 0, 0, 0 ]));
test_eval!(le32_256: "le32[256]"         => Ok(vec![ 0, 1, 0, 0 ]));
test_eval!(le32_all: "le32[4294967295]"  => Ok(vec![ 255, 255, 255, 255 ]));

test_eval!(repeat_1:  "x1(AB)"     => Ok(vec![ 0xAB; 1 ]));
test_eval!(repeat_3:  "x3(AB)"     => Ok(vec![ 0xAB; 3 ]));
test_eval!(repeat_11: "x11(AB)"    => Ok(vec![ 0xAB; 11 ]));
