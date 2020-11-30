// top-level forms
test_eval!(top_0:    "[0]"    => Ok(vec![ 0 ]));
test_eval!(top_50:   "[50]"   => Ok(vec![ 50 ]));
test_eval!(top_255:  "[255]"  => Ok(vec![ 255 ]));
test_eval!(top_256:  "[256]"  => Err(String::from("Decimal number ‘256’ does not fit in one byte")));
test_eval!(top_500:  "[500]"  => Err(String::from("Decimal number ‘500’ does not fit in one byte")));

// be16 function
test_eval!(be16_0:   "be16[0]"      => Ok(vec![ 0, 0 ]));
test_eval!(be16_50:  "be16[50]"     => Ok(vec![ 0, 50 ]));
test_eval!(be16_255: "be16[255]"    => Ok(vec![ 0, 255 ]));
test_eval!(be16_256: "be16[256]"    => Ok(vec![ 1, 0 ]));
test_eval!(be16_all: "be16[65535]"  => Ok(vec![ 255, 255 ]));
test_eval!(be16_err: "be16[65536]"  => Err(String::from("Decimal number ‘65536’ is too big for target")));

// le16 function
test_eval!(le16_0:   "le16[0]"      => Ok(vec![ 0, 0 ]));
test_eval!(le16_50:  "le16[50]"     => Ok(vec![ 50, 0 ]));
test_eval!(le16_255: "le16[255]"    => Ok(vec![ 255, 0 ]));
test_eval!(le16_256: "le16[256]"    => Ok(vec![ 0, 1 ]));
test_eval!(le16_all: "le16[65535]"  => Ok(vec![ 255, 255 ]));
test_eval!(le16_err: "le16[65536]"  => Err(String::from("Decimal number ‘65536’ is too big for target")));

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

// be64 function
test_eval!(be64_0:   "be64[0]"                     => Ok(vec![ 0, 0, 0, 0, 0, 0, 0, 0 ]));
test_eval!(be64_50:  "be64[50]"                    => Ok(vec![ 0, 0, 0, 0, 0, 0, 0, 50 ]));
test_eval!(be64_255: "be64[255]"                   => Ok(vec![ 0, 0, 0, 0, 0, 0, 0, 255 ]));
test_eval!(be64_256: "be64[256]"                   => Ok(vec![ 0, 0, 0, 0, 0, 0, 1, 0 ]));
test_eval!(be64_all: "be64[18446744073709551615]"  => Ok(vec![ 255, 255, 255, 255, 255, 255, 255, 255 ]));

// le64 function
test_eval!(le64_0:   "le64[0]"                     => Ok(vec![ 0, 0, 0, 0, 0, 0, 0, 0 ]));
test_eval!(le64_50:  "le64[50]"                    => Ok(vec![ 50, 0, 0, 0, 0, 0, 0, 0 ]));
test_eval!(le64_255: "le64[255]"                   => Ok(vec![ 255, 0, 0, 0, 0, 0, 0, 0 ]));
test_eval!(le64_256: "le64[256]"                   => Ok(vec![ 0, 1, 0, 0, 0, 0, 0, 0 ]));
test_eval!(le64_all: "le64[18446744073709551615]"  => Ok(vec![ 255, 255, 255, 255, 255, 255, 255, 255 ]));
