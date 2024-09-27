use lazy_link::lazy_link;

#[lazy_link(resolver = "my_resolver")]
extern "C" {
    static MY_VAR: &'static ();
    fn method_01();
}

fn main() {}
