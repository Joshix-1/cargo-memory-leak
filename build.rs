use std::convert::Into;

type Colour = [u8; 3];
type Symbol = char;
type List<T> = Option<Box<[T]>>;
type MatchSymbol = (
    (Symbol, Symbol, Symbol),
    (Symbol, (), Symbol),
    (Symbol, Symbol, Symbol),
);
type DestSymbol = (
    (Option<Symbol>, Option<Symbol>, Option<Symbol>),
    (Option<Symbol>, Option<Symbol>, Option<Symbol>),
    (Option<Symbol>, Option<Symbol>, Option<Symbol>),
);

#[derive(Debug)]
struct ConfigData {
    grid: Grid,
    types: List<ConcreteFieldType>,
}

#[derive(Debug)]
struct Grid {
    width: u16,
    height: u16,
}

#[derive(Debug)]
struct Type {
    symbol: Symbol,
    parent_type: Option<&'static Type>,
}

#[derive(Debug)]
struct ConcreteFieldType {
    parent_type: Type,
    name: Box<str>,
    colours: List<Colour>,
    behaviours: List<Behaviour>,
    symbol: Option<Symbol>,
}

#[derive(Debug)]
struct Behaviour {
    from: MatchSymbol,
    to: DestSymbol,
}

const ANY_TYPE: Type = Type { symbol: '*', parent_type: None };
const VOID_TYPE: Type = Type { symbol: 'v', parent_type: Some(&ANY_TYPE) };
const SOLID_TYPE: Type = Type { symbol: '#', parent_type: Some(&ANY_TYPE) };
const GAS_TYPE: Type = Type { symbol: '_', parent_type: Some(&ANY_TYPE) };

fn main() -> Result<(), String> {
    let data: ConfigData = ConfigData {
        grid: Grid {
            width: 200,
            height: 150,
        },
        types: vec![
            ConcreteFieldType {
                name: "air".into(),
                parent_type: GAS_TYPE,
                behaviours: None,
                symbol: None,
                colours: vec![[0, 0, 0]].into_boxed_slice().into(),
            },
            ConcreteFieldType {
                name: "wood".into(),
                colours: vec![[222, 184, 135]].into_boxed_slice().into(),
                symbol: None,
                parent_type: SOLID_TYPE,
                behaviours: None,
            },
            ConcreteFieldType {
                name: "sand".into(),
                colours: vec![
                    [255, 20, 147],
                    [255, 102, 179],
                    [255, 163, 194],
                    [255, 77, 148],
                    [255, 133, 149],
                    [255, 128, 161],
                    [255, 177, 173],
                    [255, 180, 229],
                ]
                .into_boxed_slice()
                .into(),
                symbol: None,
                parent_type: SOLID_TYPE,
                behaviours: vec![Behaviour {
                    from: (('*', '*', '*'), ('*', (), '*'), ('*', '*', '*')),
                    to: ((None, None, None), (None, None, None), (None, None, None)),
                }]
                .into_boxed_slice()
                .into(),
            },
        ]
        .into_boxed_slice()
        .into(),
    };

    println!("{data:?}");

    Ok(())
}
