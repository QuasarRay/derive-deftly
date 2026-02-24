use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(something = "Box<char>")]
struct DataType {
    foo: u8,
    bar: Vec<String>,
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    fn unspanned_unimpl() {
      $(
        let _: $ftype = (42).$<into>();
      )
    }

    fn spanned_unimpl() {
      $(
        let _: $ftype = (42).${paste_spanned $fname into}();
      )
    }
}

fn main() {}
