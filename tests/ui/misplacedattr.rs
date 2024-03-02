use derive_adhoc::Deftly;

#[derive(Deftly)]
struct Struct {
    #[derive_adhoc(Lemons)]
    field: usize,
}

#[derive(Deftly)]
struct Tuple(
    #[derive_adhoc(Limes)] //
    usize,
);

#[derive(Deftly)]
union Union {
    #[derive_adhoc(Oranges)]
    variant: usize,
}

#[derive(Deftly)]
enum Apples {
    #[derive_adhoc(Apples)]
    Apples {
        #[derive_adhoc(OnlyThePreviousMisplacedAttributeIsReported)]
        apples: String,
    },
}

#[derive(Deftly)]
enum Pears {
    Pears {
        #[derive_adhoc(Pears)]
        pears: String,
    },
}

#[derive(Deftly)]
enum Quince {
    Quince(
        #[derive_adhoc(Quince)] //
        String,
    ),
}

struct Const<const X: i32>;

#[derive(Deftly)]
struct Ludicrous {
    field: Const<
        {
            // accepted due to https://github.com/rust-lang/rust/issues/119116
            #[derive_adhoc(Silly)]
            #[adhoc(foolish)]
            mod inner {}

            42
        },
    >,
}

fn main() {}
