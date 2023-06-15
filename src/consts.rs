pub const MSG_END_TAG: &[u8] = b"end\0msg\0";
pub const PACKET_MAX_BUF: usize = 256;

#[cfg(test)]
pub const TEST_SEED: [u8; 32] = [
    93, 23, 248, 92, 42, 131, 233, 89, 76, 23, 48, 94, 29, 014, 212, 43, 87, 32, 248, 79, 24, 243,
    243, 97, 12, 3, 58, 186, 123, 75, 168, 13,
];
#[cfg(test)]
pub const TEST_STRING: &str = "alpha test string";
