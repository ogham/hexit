// 1-byte bits
test_eval!(bits_0:          "[b0]" => Ok(vec![ 0 ]));
test_eval!(bits_1:          "[b1]" => Ok(vec![ 1 ]));
test_eval!(bits_2:         "[b10]" => Ok(vec![ 2 ]));
test_eval!(bits_3:         "[b11]" => Ok(vec![ 3 ]));
test_eval!(bits_03:       "[b011]" => Ok(vec![ 3 ]));
test_eval!(bits_255: "[b11111111]" => Ok(vec![ 255 ]));
test_eval!(bits_0s:  "[b00000000]" => Ok(vec![ 0 ]));

// 1-byte bits, extended
test_eval!(bits_be16s: "be16[b11111110]" => Ok(vec![ 0, 254 ]));
test_eval!(bits_le16s: "le16[b11111110]" => Ok(vec![ 254, 0 ]));
test_eval!(bits_be32s: "be32[b11111110]" => Ok(vec![ 0, 0, 0, 254 ]));
test_eval!(bits_le32s: "le32[b11111110]" => Ok(vec![ 254, 0, 0, 0 ]));
test_eval!(bits_be64s: "be64[b11111110]" => Ok(vec![ 0, 0, 0, 0, 0, 0, 0, 254 ]));
test_eval!(bits_le64s: "le64[b11111110]" => Ok(vec![ 254, 0, 0, 0, 0, 0, 0, 0 ]));

// 2-byte bits
test_eval!(bits_be16_0: "be16[b00110000110011]" => Ok(vec![ 0x0C, 0x33 ]));
test_eval!(bits_be16: "be16[b1100110000110011]" => Ok(vec![ 0xCC, 0x33 ]));
test_eval!(bits_le16: "le16[b1100110000110011]" => Ok(vec![ 0x33, 0xCC ]));

// 2-byte bits, extended
test_eval!(bits_be32m: "be32[b1100110000110011]" => Ok(vec![ 0, 0, 0xCC, 0x33 ]));
test_eval!(bits_le32m: "le32[b1100110000110011]" => Ok(vec![ 0x33, 0xCC, 0, 0 ]));
test_eval!(bits_be64m: "be64[b1100110000110011]" => Ok(vec![ 0, 0, 0, 0, 0, 0, 0xCC, 0x33 ]));
test_eval!(bits_le64m: "le64[b1100110000110011]" => Ok(vec![ 0x33, 0xCC, 0, 0, 0, 0, 0, 0 ]));

// 4-byte bits
test_eval!(bits_be32_0: "be32[b001100001100110101010110101010]" => Ok(vec![ 0x0C, 0x33, 0x55, 0xAA ]));
test_eval!(bits_be32: "be32[b11001100001100110101010110101010]" => Ok(vec![ 0xCC, 0x33, 0x55, 0xAA ]));
test_eval!(bits_le32: "le32[b11001100001100110101010110101010]" => Ok(vec![ 0xAA, 0x55, 0x33, 0xCC ]));

// 4-byte bits, extended
test_eval!(bits_be64l: "be64[b11001100001100110101010110101010]" => Ok(vec![ 0, 0, 0, 0, 0xCC, 0x33, 0x55, 0xAA ]));
test_eval!(bits_le64l: "le64[b11001100001100110101010110101010]" => Ok(vec![ 0xAA, 0x55, 0x33, 0xCC, 0, 0, 0, 0 ]));

// 8-byte bits
test_eval!(bits_be64_0: "be64[b00110000110011010101011010101011110000000011111111000010101010]" => {
    Ok(vec![ 0x0C, 0x33, 0x55, 0xAA, 0xF0, 0x0F, 0xF0, 0xAA ])
});

test_eval!(bits_be64: "be64[b1100110000110011010101011010101011110000000011111111000010101010]" => {
    Ok(vec![ 0xCC, 0x33, 0x55, 0xAA, 0xF0, 0x0F, 0xF0, 0xAA ])
});

test_eval!(bits_le64: "le64[b1100110000110011010101011010101011110000000011111111000010101010]" => {
    Ok(vec![ 0xAA, 0xF0, 0x0F, 0xF0, 0xAA, 0x55, 0x33, 0xCC ])
});
