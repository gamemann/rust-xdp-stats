/* CONFIG OPTIONS */
/* -------------------------------- */
// The target UDP Port to match packets on.
pub const TARGET_PORT: u16 = 8080;

// If enabled, redirects packets to AF_XDP sockets.
pub const REDIRECT: bool = true;

// If enabled, performs a FIB lookup and sets next MAC address before redirecting packet via XDP_TX.
pub const REDIRECT_FIB_LOOKUP: bool = false;

// The path to the ELF file to load with eBPF.
// Relative to $OUT_DIR env var, but you shouldn't need to change this.
pub const PATH_ELF_FILE: &str = "rust-xdp-stats";
/* -------------------------------- */
/* CONFIG OPTIONS END */
