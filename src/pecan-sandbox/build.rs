use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        sandbox_isolate_cg: { all(feature = "isolate", feature = "isolate-cg", not(feature = "nsjail")) },
        sandbox_isolate: { all(feature = "isolate", not(feature = "nsjail")) },
        sandbox_nsjail: { all(feature = "nsjail", not(feature = "isolate")) },
    }
}
