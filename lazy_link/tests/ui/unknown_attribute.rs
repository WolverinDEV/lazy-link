use lazy_link::lazy_link;

#[lazy_link(resolver = "my_resolver", unknown = "12")]
extern "C" {
    fn method_01();
}

fn main() {}
