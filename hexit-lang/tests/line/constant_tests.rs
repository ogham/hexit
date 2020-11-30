test_eval!(bgp_open:  "BGP_OPEN"  => Ok(vec![ 0x01 ]));
test_eval!(bgp_close: "BGP_CLOSE" => Err(String::from("Unknown constant ‘BGP_CLOSE’")));
