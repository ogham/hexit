use std::collections::HashMap;
use std::fmt;


/// A constants table maps the names of constants to their values.
pub struct ConstantsTable {
    map: HashMap<&'static str, Constant>,
}

/// A constant in a table, which is of variable size.
#[derive(Copy, Clone, Debug)]
pub enum Constant {

    /// A constant that’s one byte long.
    Eight(u8),

    /// A constant that’s two bytes long.
    Sixteen(u16),
}

impl ConstantsTable {

    /// Looks up the value of a constant using its name, returning an error if
    /// no such constant exists.
    pub fn lookup(&self, name: &str) -> Result<Constant, UnknownConstant> {
        match self.map.get(name) {
            Some(value) => Ok(*value),
            None        => Err(UnknownConstant { name: name.into() }),
        }
    }

    /// Returns an iterator that yields every known constant’s name and value.
    pub fn all(&self) -> impl Iterator<Item=(&'static str, Constant)> + '_ {
        self.map.iter().map(|(&a, &b)| (a, b))
    }
}

impl ConstantsTable {

    /// Creates a new empty constants table.
    pub fn empty() -> Self {
        Self { map: HashMap::new() }
    }

    /// Creates a new constants table using the built-in set of data.
    pub fn builtin_set() -> Self {
        let mut map = HashMap::with_capacity(50);


        // BGP stuff
        // https://www.iana.org/assignments/bgp-parameters/bgp-parameters.xhtml

        // BGP message types
        map.insert("BGP_OPEN",          Constant::Eight(1));
        map.insert("BGP_UPDATE",        Constant::Eight(2));
        map.insert("BGP_NOTIFICATION",  Constant::Eight(3));
        map.insert("BGP_KEEPALIVE",     Constant::Eight(4));
        map.insert("BGP_ROUTE_REFRESH", Constant::Eight(5));


        // DNS stuff
        // https://www.iana.org/assignments/dns-parameters/dns-parameters.xhtml

        // DNS message types
        map.insert("DNS_QUERY",  Constant::Sixteen(0x0100));

        // DNS classes
        map.insert("DNS_IN",     Constant::Sixteen(1));

        // DNS record types
        map.insert("DNS_A",      Constant::Sixteen( 1));
        map.insert("DNS_NS",     Constant::Sixteen( 2));
        map.insert("DNS_CNAME",  Constant::Sixteen( 5));
        map.insert("DNS_SOA",    Constant::Sixteen( 6));
        map.insert("DNS_PTR",    Constant::Sixteen(12));
        map.insert("DNS_HINFO",  Constant::Sixteen(13));
        map.insert("DNS_MINFO",  Constant::Sixteen(14));
        map.insert("DNS_MX",     Constant::Sixteen(15));
        map.insert("DNS_TXT",    Constant::Sixteen(16));
        map.insert("DNS_GPOS",   Constant::Sixteen(27));
        map.insert("DNS_AAAA",   Constant::Sixteen(28));
        map.insert("DNS_SRV",    Constant::Sixteen(33));


        // Ethernet stuff
        // https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml

        // Ethernet types (EtherTypes)
        map.insert("ETHERTYPE_IPv4",         Constant::Sixteen(0x0800));
        map.insert("ETHERTYPE_ARP",          Constant::Sixteen(0x0806));
        map.insert("ETHERTYPE_WAKE_ON_LAN",  Constant::Sixteen(0x0842));
        map.insert("ETHERTYPE_IPV6",         Constant::Sixteen(0x86DD));


        // Gzip stuff
        // http://www.gzip.org/format.txt

        // Gzip compression methods
        map.insert("GZIP_DEFLATE",  Constant::Eight(0x08));

        // Gzip compression flags
        map.insert("GZIP_SLOWEST",  Constant::Eight(0x02));
        map.insert("GZIP_FASTEST",  Constant::Eight(0x04));

        // Gzip flags
        map.insert("GZIP_FTEXT",    Constant::Eight(0x01));
        map.insert("GZIP_FHCRC",    Constant::Eight(0x02));
        map.insert("GZIP_FEXTRA",   Constant::Eight(0x04));
        map.insert("GZIP_FNAME",    Constant::Eight(0x08));
        map.insert("GZIP_FCOMMENT", Constant::Eight(0x10));

        // Gzip OSes
        map.insert("GZIP_FAT",      Constant::Eight( 0));
        map.insert("GZIP_UNIX",     Constant::Eight( 3));
        map.insert("GZIP_NT",       Constant::Eight(11));


        // ICMP stuff
        // https://www.iana.org/assignments/icmp-parameters/icmp-parameters.xhtml

        // ICMP message types
        map.insert("ICMP_ECHO_REPLY",               Constant::Eight( 0));
        map.insert("ICMP_DESTINATION_UNREACHABLE",  Constant::Eight( 2));
        map.insert("ICMP_REDIRECT",                 Constant::Eight( 5));
        map.insert("ICMP_ECHO",                     Constant::Eight( 8));
        map.insert("ICMP_ROUTER_ADVERTISEMENT",     Constant::Eight( 9));
        map.insert("ICMP_ROUTER_SOLICITATION",      Constant::Eight(10));
        map.insert("ICMP_TIME_EXCEEDED",            Constant::Eight(11));
        map.insert("ICMP_PARAMETER_PROBLEM",        Constant::Eight(12));
        map.insert("ICMP_TIMESTAMP_REQUEST",        Constant::Eight(13));
        map.insert("ICMP_TIMESTAMP_REPLY",          Constant::Eight(14));
        map.insert("ICMP_ADDRESS_MASK_REQUEST",     Constant::Eight(17));
        map.insert("ICMP_ADDRESS_MASK_REPLY",       Constant::Eight(18));


        // IP stuff
        // https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml

        // IP protocols [/etc/protocols]
        map.insert("IP_ICMP",  Constant::Eight(  1));
        map.insert("IP_IGMP",  Constant::Eight(  2));
        map.insert("IP_TCP",   Constant::Eight(  6));
        map.insert("IP_UDP",   Constant::Eight( 17));
        map.insert("IP_SCTP",  Constant::Eight(132));


        // TCP stuff
        // https://www.iana.org/assignments/tcp-parameters/tcp-parameters.xhtml

        // TCP flags
        map.insert("TCP_FIN",  Constant::Sixteen(0x0001));
        map.insert("TCP_SYN",  Constant::Sixteen(0x0002));
        map.insert("TCP_RST",  Constant::Sixteen(0x0004));
        map.insert("TCP_PSH",  Constant::Sixteen(0x0008));
        map.insert("TCP_ACK",  Constant::Sixteen(0x0010));
        map.insert("TCP_URG",  Constant::Sixteen(0x0020));
        map.insert("TCP_ECN",  Constant::Sixteen(0x0040));
        map.insert("TCP_CWR",  Constant::Sixteen(0x0080));

        Self { map }
    }
}


/// The error returned when looking up a constant that is not in the table.
#[derive(PartialEq, Debug)]
pub struct UnknownConstant {
    name: String,
}

impl fmt::Display for UnknownConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown constant ‘{}’", self.name)
    }
}
