use lazy_link::lazy_link;

#[lazy_link]
extern "C" {
    fn method_01();
}

fn main() {}
