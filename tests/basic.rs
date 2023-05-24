use num_enum::{IntoPrimitive, TryFromPrimitive};
use pretty_assertions::assert_eq;

use memento::{Alloc, Error, Stat, UseCase};

#[derive(TryFromPrimitive, IntoPrimitive, Default, Debug, Ord, Eq, PartialEq, PartialOrd)]
#[repr(u32)]
enum MyUseCase {
    #[default]
    None,
    JsonPayload,
    UserProfile,
    ConfigFile,
}

impl UseCase for MyUseCase {}

type Allocator = Alloc<MyUseCase>;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

macro_rules! get {
    ($usecase:ident) => {
        ALLOCATOR
            .with_recorder(|recorder| Ok(recorder.get(MyUseCase::$usecase).current))
            .unwrap()
    };
}

#[test]
#[allow(unused_variables)]
fn basic() {
    let before = get!(None);
    let foo = vec!["foo".to_owned(); 200];

    assert_eq!(get!(None), before + 5400);

    let guard = ALLOCATOR.with_usecase(MyUseCase::JsonPayload);
    let bar = vec!["bar".to_owned(); 300];
    drop(guard);

    assert_eq!(get!(None), before + 5400);
    assert_eq!(get!(JsonPayload), 8100);
    drop(bar);

    assert_eq!(get!(None), before + 5400);
    assert_eq!(get!(JsonPayload), 0);

    ALLOCATOR
        .with_recorder(|recorder| {
            let mut records = Vec::new();
            recorder.flush(
                |usecase, stat| records.push((usecase, stat)),
                |err, count| {
                    if count > 0 && err != Error::CurrentUsecaseContentionRefCell {
                        panic!("unexpected error: {:?}", err);
                    }
                },
            );
            records.sort();
            assert_eq!(
                records,
                vec![
                    (
                        MyUseCase::None,
                        Stat {
                            current: before + 5400,
                            peak: before + 10613,
                            total: before + 40192,
                        },
                    ),
                    (
                        MyUseCase::JsonPayload,
                        Stat {
                            current: 0,
                            peak: 8100,
                            total: 8100,
                        },
                    ),
                ]
            );
            Ok(())
        })
        .unwrap();
}
