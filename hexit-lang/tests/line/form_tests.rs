test_eval!(localhost: "[127.0.0.1]"            => Ok(vec![ 127, 0, 0, 1 ]));
test_eval!(broadcast: "[255.255.255.255]"      => Ok(vec![ 255, 255, 255, 255 ]));
test_eval!(ipv6:      "[::1]"                  => Ok(vec![ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 ]));

test_eval!(timestamp: "[2017-12-31T21:36:45]" => Ok(vec![ 0x5A, 0x49, 0x58, 0x6D ]));
