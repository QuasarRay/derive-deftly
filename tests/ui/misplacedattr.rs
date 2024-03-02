use derive_deftly::Deftly;

#[derive(Deftly)]
struct Struct {
    #[derive_deftly(Lemons)]
    field: usize,
}

#[derive(Deftly)]
struct Tuple(
    #[derive_deftly(Limes)] //
    usize,
);

#[derive(Deftly)]
union Union {
    #[derive_deftly(Oranges)]
    variant: usize,
}

#[derive(Deftly)]
enum Apples {
    #[derive_deftly(Apples)]
    Apples {
        #[derive_deftly(OnlyThePreviousMisplacedAttributeIsReported)]
        apples: String,
    },
}

#[derive(Deftly)]
enum Pears {
    Pears {
        #[derive_deftly(Pears)]
        pears: String,
    },
}

#[derive(Deftly)]
enum Quince {
    Quince(
        #[derive_deftly(Quince)] //
        String,
    ),
}

struct Const<const X: i32>;

#[derive(Deftly)]
struct Ludicrous {
    field: Const<
        {
            // accepted due to https://github.com/rust-lang/rust/issues/119116
            #[derive_deftly(Silly)]
            #[deftly(foolish)]
            mod inner {}

            42
        },
    >,
}

fn main() {}
