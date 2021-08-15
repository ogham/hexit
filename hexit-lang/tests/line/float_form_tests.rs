// Use one of these sites to check floats:
// - <https://www.h-schmidt.net/FloatConverter/IEEE754.html>
// - <https://kayru.org/articles/float/>


// ---- be32 ----

// halving fractions
test_eval!(float_be32_0_5:          "be32[f0.5]" => Ok(vec![ 0x3f, 0x00, 0x00, 0x00 ]));
test_eval!(float_be32_0_25:        "be32[f0.25]" => Ok(vec![ 0x3e, 0x80, 0x00, 0x00 ]));
test_eval!(float_be32_0_125:      "be32[f0.125]" => Ok(vec![ 0x3e, 0x00, 0x00, 0x00 ]));
test_eval!(float_be32_0_0625:    "be32[f0.0625]" => Ok(vec![ 0x3d, 0x80, 0x00, 0x00 ]));

// inexact fractions
test_eval!(float_be32_0_1:          "be32[f0.1]" => Ok(vec![ 0x3d, 0xcc, 0xcc, 0xcd ]));
test_eval!(float_be32_0_123: "be32[f0.00001234]" => Ok(vec![ 0x37, 0x4f, 0x07, 0xe5 ]));

// mixed numbers
test_eval!(float_be32_100:      "be32[f100.001]" => Ok(vec![ 0x42, 0xc8, 0x00, 0x83 ]));
test_eval!(float_be32_20000:  "be32[f2000.0002]" => Ok(vec![ 0x44, 0xfa, 0x00, 0x02 ]));
test_eval!(float_be32_n33:       "be32[f-33.33]" => Ok(vec![ 0xc2, 0x05, 0x51, 0xec ]));

// exponents
test_eval!(float_be32_exp:       "be32[f2.7e18]" => Ok(vec![ 0x5e, 0x15, 0xe1, 0x4f ]));
test_eval!(float_be32_nexp:    "be32[f-2.7e-18]" => Ok(vec![ 0xa2, 0x47, 0x39, 0x8f ]));
test_eval!(float_be32: "be32[f1.3211836173E+19]" => Ok(vec![ 0x5f, 0x37, 0x59, 0xdf ]));

// special cases
test_eval!(float_be32_nan:          "be32[fNaN]" => Ok(vec![ 0x7f, 0xc0, 0x00, 0x00 ]));
test_eval!(float_be32_p0:            "be32[f+0]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be32_n0:            "be32[f-0]" => Ok(vec![ 0x80, 0x00, 0x00, 0x00 ]));
test_eval!(float_be32_pinf:         "be32[finf]" => Ok(vec![ 0x7f, 0x80, 0x00, 0x00 ]));
test_eval!(float_be32_ninf:        "be32[f-inf]" => Ok(vec![ 0xff, 0x80, 0x00, 0x00 ]));


// ---- le32 ----

// halving fractions
test_eval!(float_le32_0_5:          "le32[f0.5]" => Ok(vec![ 0x00, 0x00, 0x00, 0x3f ]));
test_eval!(float_le32_0_25:        "le32[f0.25]" => Ok(vec![ 0x00, 0x00, 0x80, 0x3e ]));
test_eval!(float_le32_0_125:      "le32[f0.125]" => Ok(vec![ 0x00, 0x00, 0x00, 0x3e ]));
test_eval!(float_le32_0_0625:    "le32[f0.0625]" => Ok(vec![ 0x00, 0x00, 0x80, 0x3d ]));

// inexact fractions
test_eval!(float_le32_0_1:          "le32[f0.1]" => Ok(vec![ 0xcd, 0xcc, 0xcc, 0x3d ]));
test_eval!(float_le32_0_123: "le32[f0.00001234]" => Ok(vec![ 0xe5, 0x07, 0x4f, 0x37 ]));

// mixed numbers
test_eval!(float_le32_100:      "le32[f100.001]" => Ok(vec![ 0x83, 0x00, 0xc8, 0x42 ]));
test_eval!(float_le32_20000:  "le32[f2000.0002]" => Ok(vec![ 0x02, 0x00, 0xfa, 0x44 ]));
test_eval!(float_le32_n33:       "le32[f-33.33]" => Ok(vec![ 0xec, 0x51, 0x05, 0xc2 ]));

// exponents
test_eval!(float_le32_exp:       "le32[f2.7e18]" => Ok(vec![ 0x4f, 0xe1, 0x15, 0x5e ]));
test_eval!(float_le32_nexp:    "le32[f-2.7e-18]" => Ok(vec![ 0x8f, 0x39, 0x47, 0xa2 ]));
test_eval!(float_le32: "le32[f1.3211836173E+19]" => Ok(vec![ 0xdf, 0x59, 0x37, 0x5f ]));

// special cases
test_eval!(float_le32_nan:          "le32[fNaN]" => Ok(vec![ 0x00, 0x00, 0xc0, 0x7f ]));
test_eval!(float_le32_p0:            "le32[f+0]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_le32_n0:            "le32[f-0]" => Ok(vec![ 0x00, 0x00, 0x00, 0x80 ]));
test_eval!(float_le32_pinf:         "le32[finf]" => Ok(vec![ 0x00, 0x00, 0x80, 0x7f ]));
test_eval!(float_le32_ninf:        "le32[f-inf]" => Ok(vec![ 0x00, 0x00, 0x80, 0xff ]));


// ---- be64 ----

// halving fractions
test_eval!(float_be64_0_5:          "be64[f0.5]" => Ok(vec![ 0x3f, 0xe0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_0_25:        "be64[f0.25]" => Ok(vec![ 0x3f, 0xd0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_0_125:      "be64[f0.125]" => Ok(vec![ 0x3f, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_0_0625:    "be64[f0.0625]" => Ok(vec![ 0x3f, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));

// inexact fractions
test_eval!(float_be64_0_1:          "be64[f0.1]" => Ok(vec![ 0x3f, 0xb9, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9a ]));
test_eval!(float_be64_0_123: "be64[f0.00001234]" => Ok(vec![ 0x3e, 0xe9, 0xe0, 0xfc, 0xaf, 0x93, 0x80, 0xfc ]));

// mixed numbers
test_eval!(float_be64_100:      "be64[f100.001]" => Ok(vec![ 0x40, 0x59, 0x00, 0x10, 0x62, 0x4d, 0xd2, 0xf2 ]));
test_eval!(float_be64_20000:  "be64[f2000.0002]" => Ok(vec![ 0x40, 0x9f, 0x40, 0x00, 0x34, 0x6d, 0xc5, 0xd6]));
test_eval!(float_be64_n33:       "be64[f-33.33]" => Ok(vec![ 0xc0, 0x40, 0xaa, 0x3d, 0x70, 0xa3, 0xd7, 0x0a ]));

// exponents
test_eval!(float_be64_exp:       "be64[f2.7e18]" => Ok(vec![ 0x43, 0xc2, 0xbc, 0x29, 0xd8, 0xee, 0xc7, 0x00 ]));
test_eval!(float_be64_nexp:    "be64[f-2.7e-18]" => Ok(vec![ 0xbc, 0x48, 0xe7, 0x31, 0xdb, 0x42, 0x41, 0xc1 ]));
test_eval!(float_be64: "be64[f1.3211836173E+19]" => Ok(vec![ 0x43, 0xe6, 0xeb, 0x3b, 0xe0, 0x00, 0x4a, 0x48 ]));

// special cases
test_eval!(float_be64_nan:          "be64[fNaN]" => Ok(vec![ 0x7f, 0xf8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_p0:            "be64[f+0]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_n0:            "be64[f-0]" => Ok(vec![ 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_pinf:         "be64[finf]" => Ok(vec![ 0x7f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_be64_ninf:        "be64[f-inf]" => Ok(vec![ 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));


// ---- le64 ----

// halving fractions
test_eval!(float_le64_0_5:          "le64[f0.5]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x3f ]));
test_eval!(float_le64_0_25:        "le64[f0.25]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd0, 0x3f ]));
test_eval!(float_le64_0_125:      "le64[f0.125]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x3f ]));
test_eval!(float_le64_0_0625:    "le64[f0.0625]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x3f ]));

// inexact fractions
test_eval!(float_le64_0_1:          "le64[f0.1]" => Ok(vec![ 0x9a, 0x99, 0x99, 0x99, 0x99, 0x99, 0xb9, 0x3f ]));
test_eval!(float_le64_0_123: "le64[f0.00001234]" => Ok(vec![ 0xfc, 0x80, 0x93, 0xaf, 0xfc, 0xe0, 0xe9, 0x3e ]));

// mixed numbers
test_eval!(float_le64_100:      "le64[f100.001]" => Ok(vec![ 0xf2, 0xd2, 0x4d, 0x62, 0x10, 0x00, 0x59, 0x40 ]));
test_eval!(float_le64_20000:  "le64[f2000.0002]" => Ok(vec![ 0xd6, 0xc5, 0x6d, 0x34, 0x00, 0x40, 0x9f, 0x40]));
test_eval!(float_le64_n33:       "le64[f-33.33]" => Ok(vec![ 0x0a, 0xd7, 0xa3, 0x70, 0x3d, 0xaa, 0x40, 0xc0 ]));

// exponents
test_eval!(float_le64_exp:       "le64[f2.7e18]" => Ok(vec![ 0x00, 0xc7, 0xee, 0xd8, 0x29, 0xbc, 0xc2, 0x43 ]));
test_eval!(float_le64_nexp:    "le64[f-2.7e-18]" => Ok(vec![ 0xc1, 0x41, 0x42, 0xdb, 0x31, 0xe7, 0x48, 0xbc ]));
test_eval!(float_le64: "le64[f1.3211836173E+19]" => Ok(vec![ 0x48, 0x4a, 0x00, 0xe0, 0x3b, 0xeb, 0xe6, 0x43 ]));

// special cases
test_eval!(float_le64_nan:          "le64[fNaN]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x7f ]));
test_eval!(float_le64_p0:            "le64[f+0]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]));
test_eval!(float_le64_n0:            "le64[f-0]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80 ]));
test_eval!(float_le64_pinf:         "le64[finf]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f ]));
test_eval!(float_le64_ninf:        "le64[f-inf]" => Ok(vec![ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xff ]));


// ---- errors ----

test_eval!(top_level_float:     "[f1.2]" => Err(String::from("Floating-point number ‘1.2’ does not fit in one byte")));
test_eval!(be16_float:      "be16[f1.2]" => Err(String::from("Floating-point number ‘1.2’ is too big for target")));
test_eval!(le16_float:      "le16[f1.2]" => Err(String::from("Floating-point number ‘1.2’ is too big for target")));