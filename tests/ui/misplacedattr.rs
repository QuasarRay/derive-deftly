use derive_adhoc::Adhoc;

#[derive(Adhoc)]
struct Struct {
    #[derive_adhoc(Lemons)]
    field: usize,
}

#[derive(Adhoc)]
struct Tuple(
    #[derive_adhoc(Limes)] //
    usize,
);

#[derive(Adhoc)]
union Union {
    #[derive_adhoc(Oranges)]
    variant: usize,
}

#[derive(Adhoc)]
enum Apples {
    #[derive_adhoc(Apples)]
    Apples {
        #[derive_adhoc(OnlyThePreviousMisplacedAttributeIsReported)]
        apples: String,
    },
}

#[derive(Adhoc)]
enum Pears {
    Pears {
        #[derive_adhoc(Pears)]
        pears: String,
    },
}

#[derive(Adhoc)]
enum Quince {
    Quince(
        #[derive_adhoc(Quince)] //
        String,
    ),
}

struct Const<const X: i32>;

#[derive(Adhoc)]
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
